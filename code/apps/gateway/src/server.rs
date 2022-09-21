use std::{
    collections::HashMap,
    sync::Arc,
};

use actix::prelude::{
    Actor,
    Context,
    Handler,
};
use uuid::Uuid;

use crate::{
    config::CommsMode,
    messages::{
        BroadcastServerMessage,
        ClientMessage,
        Connect,
        Disconnect,
        Heartbeat,
        ServerDisconnect,
        ServerMessage,
    },
};

type Socket = actix::prelude::Recipient<crate::messages::WsMessage>;
type SharedSessionMap = std::sync::Arc<std::sync::RwLock<HashMap<String, (String, Socket)>>>;

pub struct Server {
    config: crate::config::Config,
    instance: String,
    sessions: SharedSessionMap,
    redis: std::sync::Arc<redis::Client>,
    nats_js: Option<std::sync::Arc<nats::jetstream::JetStream>>,

    #[allow(dead_code)]
    redis_thread: std::thread::JoinHandle<()>,
    #[allow(dead_code)]
    stats_reporting_thread: std::option::Option<std::thread::JoinHandle<()>>,
}

impl Server {
    pub fn new(
        config: crate::config::Config,
        instance: String,
        redis: redis::Client,
        nats_js: Option<nats::jetstream::JetStream>,
    ) -> Self {
        let redis_connection_arc = std::sync::Arc::new(redis);
        let session_map_arc: SharedSessionMap =
            std::sync::Arc::new(std::sync::RwLock::new(HashMap::<String, (String, Socket)>::new()));

        let rt = Self::start_redis_thread(
            instance.clone(),
            config.group_id.clone(),
            redis_connection_arc.clone(),
            session_map_arc.clone(),
        );
        let srt = match config.server.stats_interval_sec {
            | Some(v) => Some(Self::start_stats_reporting_thread(
                instance.clone(),
                v.into(),
                session_map_arc.clone(),
            )),
            | None => None,
        };

        Server {
            config,
            instance,
            sessions: session_map_arc,
            redis: redis_connection_arc,
            nats_js: match nats_js {
                | Some(v) => Some(Arc::new(v)),
                | None => None,
            },
            redis_thread: rt,
            stats_reporting_thread: srt,
        }
    }

    pub fn make_key(&self, connection: &str) -> String {
        format!(
            "hydrogen:{}:i2c:{}:{}",
            self.config.group_id,
            self.instance,
            connection.to_string()
        )
    }

    pub fn make_reverse_key(&self, connection: &str) -> String {
        format!("hydrogen:{}:c2i:{}", self.config.group_id, connection)
    }

    /// Function will start a new thread and return it's JoinHandle. This thread
    /// will listen to the redis pub/sub channel that's relevant
    /// for this instance and process it's messages.
    fn start_redis_thread(
        instance_id: String,
        group_id: String,
        redis_conn: std::sync::Arc<redis::Client>,
        sessions: SharedSessionMap,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let mut conn = redis_conn.get_connection().unwrap();
            let mut ps = conn.as_pubsub();
            let thread_instance_id = instance_id.clone();
            ps.subscribe(format!("hydrogen:{}:broadcast", group_id)).unwrap();
            ps.subscribe(format!("hydrogen:{}", instance_id)).unwrap();

            loop {
                let mut safecall = || -> Result<(), Box<dyn std::error::Error>> {
                    let msg = ps.get_message()?;
                    let pl: String = msg.get_payload()?;
                    let payload: hydrogen_bus::redis::Message = serde_json::from_str(&pl)?;
                    match payload {
                        | hydrogen_bus::redis::Message::SBroadcast { time: _, message } => {
                            crate::logger::LogMessage::now(&thread_instance_id, crate::logger::Data::Event {
                                data: crate::logger::Event::ServerBroadcastMessagePost {},
                            });

                            for s in sessions.read()?.iter() {
                                s.1 .1.do_send(crate::messages::WsMessage::Message {
                                    endpoint: s.1 .0.to_owned(),
                                    message: message.clone(),
                                });
                            }
                            Ok(())
                        },
                        // Handles messages for connections this instance owns.
                        | hydrogen_bus::redis::Message::S2CMessage {
                            connection,
                            time: _,
                            message,
                        } => {
                            crate::logger::LogMessage::now(&thread_instance_id, crate::logger::Data::Event {
                                data: crate::logger::Event::ServerMessagePost {
                                    connection: &connection,
                                },
                            });
                            match sessions.read()?.get(&connection) {
                                | Some(s) => {
                                    s.1.do_send(crate::messages::WsMessage::Message {
                                        endpoint: s.0.to_owned(),
                                        message: message.clone(),
                                    });
                                    Ok(())
                                },
                                | None => Err(Box::new(crate::error::ConnectionNotFoundError::new(
                                    &connection.to_string(),
                                ))),
                            }
                        },
                        // Handles server disconnect requests for a connection this instance owns..
                        | hydrogen_bus::redis::Message::SDisconnect {
                            connection,
                            time: _,
                            reason,
                        } => {
                            crate::logger::LogMessage::now(&thread_instance_id, crate::logger::Data::Event {
                                data: crate::logger::Event::ServerDisconnect {
                                    connection: &connection,
                                    reason: &reason,
                                },
                            });
                            match sessions.read()?.get(&connection) {
                                | Some(s) => {
                                    s.1.do_send(crate::messages::WsMessage::Disconnect(reason));
                                    Ok(())
                                },
                                | None => Err(Box::new(crate::error::ConnectionNotFoundError::new(
                                    &connection.to_string(),
                                ))),
                            }
                        },
                    }
                };
                match safecall() {
                    | Ok(_) => {},
                    | Err(e) => {
                        crate::logger::LogMessage::now(&thread_instance_id, crate::logger::Data::Event {
                            data: crate::logger::Event::Error { err: &e.to_string() },
                        });
                        std::thread::sleep(std::time::Duration::from_secs(5));
                    },
                }
            }
        })
    }

    fn start_stats_reporting_thread(
        instance_id: String,
        interval: u64,
        sessions: SharedSessionMap,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || loop {
            crate::logger::LogMessage::now(&instance_id, crate::logger::Data::Interval {
                stats: crate::logger::Stats::Connections {
                    count: sessions.read().unwrap().len(),
                    connections: sessions.read().unwrap().keys().into_iter().collect(),
                },
            });
            std::thread::sleep(std::time::Duration::from_secs(interval));
        })
    }

    fn invoke_connect_route(
        &self,
        endpoint: &str,
        headers: &std::collections::HashMap<String, String>,
        message: &crate::messages::Connect,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut req = ureq::post(endpoint);
        for (k, v) in headers.iter() {
            req = req.set(k, v);
        }

        let resp = req.send_string(&serde_json::to_string(&crate::routes::ConnectRequest {
            instance_id: self.instance.clone(),
            group_id: self.config.group_id.clone(),
            endpoint: message.endpoint.clone(),
            connection_id: message.connection.clone(),
            time: message.time.clone(),
        })?)?;

        crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
            data: crate::logger::Event::ConnectRouteResponse {
                connection: &message.connection,
                response: resp.status(),
            },
        });

        match resp.status() {
            | 200 => Ok(()),
            | _ => Err(Box::new(crate::error::ConnectRouteError::new(&format!(
                "connect route error code {}",
                resp.status()
            )))),
        }
    }

    fn invoke_disconnect_route(
        &self,
        endpoint: &str,
        headers: &std::collections::HashMap<String, String>,
        message: &crate::messages::Disconnect,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut req = ureq::post(endpoint);
        for (k, v) in headers.iter() {
            req = req.set(k, v);
        }

        let resp = req.send_string(&serde_json::to_string(&crate::routes::DisconnectRequest {
            instance_id: self.instance.clone(),
            group_id: self.config.group_id.clone(),
            endpoint: message.endpoint.clone(),
            connection_id: message.connection.clone(),
            time: message.time.clone(),
        })?)?;

        crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
            data: crate::logger::Event::DisconnectRouteResponse {
                connection: &message.connection,
                response: resp.status(),
            },
        });

        match resp.status() {
            | 200 => Ok(()),
            | _ => Err(Box::new(crate::error::DisconnectRouteError::new(&format!(
                "disconnect route error code {}",
                resp.status()
            )))),
        }
    }
}

impl Actor for Server {
    type Context = Context<Self>;
}

/// Handler for the OnConnect event in which a client has been permitted for a
/// server connection and is now establishing the connection.
impl Handler<Connect> for Server {
    type Result = std::result::Result<(), u16>;

    /// This function will create a client/server map in redis for the
    /// connection (id) and this instance (id). It will also invoke the
    /// connect route if specified.
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let safecall = || -> Result<(), Box<dyn std::error::Error>> {
            crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                data: crate::logger::Event::Connect {
                    connection: &msg.connection.to_string(),
                },
            });
            self.sessions
                .write()
                .unwrap()
                .insert(msg.connection.clone(), (msg.endpoint.clone(), msg.addr.clone())); // must never be poisoned
            let key = self.make_key(&msg.connection);
            let rkey = self.make_reverse_key(&msg.connection);

            redis::pipe()
                .cmd("SET")
                .arg(&key)
                .arg(1)
                .cmd("EXPIRE")
                .arg(&key)
                .arg(30)
                .cmd("SET")
                .arg(&rkey)
                .arg(&self.instance.to_string())
                .cmd("EXPIRE")
                .arg(&rkey)
                .arg(30)
                .query::<()>(&mut self.redis.get_connection()?)?;

            match &self.config.routes.connect {
                | Some(c) => self.invoke_connect_route(&c.endpoint, &c.headers, &msg),
                | None => Ok(()),
            }?;

            Ok(())
        };
        match safecall() {
            | Ok(_) => Ok(()),
            | Err(e) => {
                crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                    data: crate::logger::Event::Error { err: &e.to_string() },
                });
                self.sessions.write().unwrap().remove(&msg.connection); // must never be poisoned
                Err(500_u16)
            },
        }
    }
}

/// Handler for disconnect events which occurr when a client or the server ends
/// the connection.
impl Handler<Disconnect> for Server {
    type Result = std::result::Result<(), u16>;

    /// This function will purge the redis client/server mappings invoke the
    /// disconnect route if specified.
    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
        let safecall = || -> Result<(), Box<dyn std::error::Error>> {
            crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                data: crate::logger::Event::Disconnect {
                    connection: &msg.connection.to_string(),
                },
            });

            let key = self.make_key(&msg.connection);
            let rkey = self.make_reverse_key(&msg.connection);
            redis::pipe()
                .cmd("DEL")
                .arg(&key)
                .cmd("DEL")
                .arg(&rkey)
                .query::<()>(&mut self.redis.get_connection()?)?;

            self.sessions.write().unwrap().remove(&msg.connection); // must never be poisoned

            match &self.config.routes.disconnect {
                | Some(c) => self.invoke_disconnect_route(&c.endpoint, &c.headers, &msg),
                | None => Ok(()),
            }
        };
        match safecall() {
            | Ok(_) => Ok(()),
            | Err(e) => {
                crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                    data: crate::logger::Event::Error { err: &e.to_string() },
                });
                Err(500_u16)
            },
        }
    }
}

/// Handler for heartbeat messages on a connection. Heartbeats are used to
/// ping/pong whether a connection is still established and a client still
/// active. It also helps to prevent timeouts for connections that are
/// established but do not see any message for a certain perdiod of time.
impl Handler<Heartbeat> for Server {
    type Result = std::result::Result<(), u16>;

    /// This function will update the expiry time on the client-to-server as
    /// well as server-to-client mappings in redis.
    fn handle(&mut self, msg: Heartbeat, _ctx: &mut Context<Self>) -> Self::Result {
        let safecall = || -> Result<(), Box<dyn std::error::Error>> {
            let key = self.make_key(&msg.connection);
            let rkey = self.make_reverse_key(&msg.connection);
            redis::pipe()
                .cmd("EXPIRE")
                .arg(&key)
                .arg(30)
                .cmd("EXPIRE")
                .arg(&rkey)
                .arg(30)
                .query::<()>(&mut self.redis.get_connection()?)?;
            Ok(())
        };
        match safecall() {
            | Ok(_) => Ok(()),
            | Err(e) => {
                crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                    data: crate::logger::Event::Error { err: &e.to_string() },
                });
                Err(500_u16)
            },
        }
    }
}

/// Handler for messages that are sent from this server towards any client.
impl Handler<ServerMessage> for Server {
    type Result = ();

    /// This function will take the message and the specified connection, lookup
    /// the instance of the gateway that holds the specified client
    /// connection and post the message into the corresponding redis pub/sub
    /// channel.
    fn handle(&mut self, msg: ServerMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let safecall = || -> Result<(), Box<dyn std::error::Error>> {
            crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                data: crate::logger::Event::ServerMessageEnqueue {
                    connection: &msg.connection.to_string(),
                },
            });

            let conn = msg.connection.clone();
            let redis_message: hydrogen_bus::redis::Message = msg.into();

            let target_instance = redis::cmd("GET")
                .arg(&self.make_reverse_key(&conn))
                .query::<String>(&mut self.redis.get_connection()?)?;
            redis::pipe()
                .publish(target_instance, serde_json::to_string(&redis_message)?)
                .query::<()>(&mut self.redis.get_connection()?)?;
            Ok(())
        };
        match safecall() {
            | Ok(_) => {},
            | Err(e) => {
                crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                    data: crate::logger::Event::Error { err: &e.to_string() },
                });
            },
        }
    }
}

/// Handler for messages that are sent from this server towards any client.
impl Handler<BroadcastServerMessage> for Server {
    type Result = ();

    /// This function will take the message and the specified connection, lookup
    /// the instance of the gateway that holds the specified client
    /// connection and post the message into the corresponding redis pub/sub
    /// channel.
    fn handle(&mut self, msg: BroadcastServerMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let safecall = || -> Result<(), Box<dyn std::error::Error>> {
            crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                data: crate::logger::Event::ServerBroadcastMessageEnqueue {},
            });

            let redis_message: hydrogen_bus::redis::Message = msg.into();

            redis::pipe()
                .publish(
                    format!("hydrogen:{}:broadcast", self.config.group_id),
                    serde_json::to_string(&redis_message)?,
                )
                .query::<()>(&mut self.redis.get_connection()?)?;
            Ok(())
        };
        match safecall() {
            | Ok(_) => {},
            | Err(e) => {
                crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                    data: crate::logger::Event::Error { err: &e.to_string() },
                });
            },
        }
    }
}

/// Handler for the event in which the server needs to end the connection to any
/// client.
impl Handler<ServerDisconnect> for Server {
    type Result = ();

    /// This function will lookup the mapped instance for the given connection
    /// in redis and post a disconnect request for the specified instance
    /// and the given connection id.
    fn handle(&mut self, msg: ServerDisconnect, _ctx: &mut Context<Self>) -> Self::Result {
        let safecall = || -> Result<(), Box<dyn std::error::Error>> {
            crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                data: crate::logger::Event::ServerDisconnect {
                    connection: &msg.connection.to_string(),
                    reason: &msg.reason,
                },
            });

            let conn = msg.connection.clone();
            let redis_message: hydrogen_bus::redis::Message = msg.into();

            let target_instance = redis::cmd("GET")
                .arg(&self.make_reverse_key(&conn))
                .query::<String>(&mut self.redis.get_connection()?)?;
            redis::pipe()
                .publish(target_instance, serde_json::to_string(&redis_message)?)
                .query::<()>(&mut self.redis.get_connection()?)?;
            Ok(())
        };
        match safecall() {
            | Ok(_) => {},
            | Err(e) => {
                crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                    data: crate::logger::Event::Error { err: &e.to_string() },
                });
            },
        }
    }
}

/// Handler for client messages the server receives.
impl Handler<ClientMessage> for Server {
    type Result = std::result::Result<(), u16>;

    /// This function will publish the message towards a message topic in a
    /// NATS/Jetstream stream.
    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let safecall = || -> Result<(), Box<dyn std::error::Error>> {
            crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                data: crate::logger::Event::ClientMessage {
                    connection: &msg.connection.to_string(),
                },
            });

            match &self.nats_js {
                | Some(v) => {
                    let stream_name = match &self.config.server.comms {
                        | CommsMode::UniServerToClient => {
                            Err(crate::error::ConfigError::new("server comms mode is uni"))
                        },
                        | CommsMode::Bidi { stream } => Ok(stream.name.clone()),
                    }?;
                    v.publish_with_options(
                        &format!("hydrogen.{}.core.v1.$client", self.config.group_id),
                        serde_json::json!(hydrogen_bus::nats::Message {
                            meta: hydrogen_bus::nats::MessageMeta {
                                id: Uuid::new_v4().to_string(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            },
                            data: hydrogen_bus::nats::ClientMessage {
                                instance_id: self.instance.clone(),
                                connection_id: msg.connection,
                                context: msg.context.into(),
                                message: msg.message,
                            }
                        })
                        .to_string(),
                        &nats::jetstream::PublishOptions {
                            expected_stream: Some(stream_name),
                            ..Default::default()
                        },
                    )?;
                },
                | None => {},
            }
            Ok(())
        };
        match safecall() {
            | Ok(_) => Ok(()),
            | Err(e) => {
                crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                    data: crate::logger::Event::Error { err: &e.to_string() },
                });
                Err(500_u16)
            },
        }
    }
}
