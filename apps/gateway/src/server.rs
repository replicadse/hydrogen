use std::collections::HashMap;

use actix::prelude::{
    Actor,
    Context,
    Handler,
};

use crate::messages::{
    ClientMessage,
    Connect,
    Disconnect,
    Heartbeat,
    ServerDisconnect,
    ServerMessage,
};

type Socket = actix::prelude::Recipient<crate::messages::WsMessage>;

pub struct Server {
    config: crate::config::Config,
    instance: String,
    sessions: std::sync::Arc<std::sync::RwLock<HashMap<String, Socket>>>,
    redis: std::sync::Arc<redis::Client>,
    nats: std::sync::Arc<nats::jetstream::JetStream>,
}

impl Server {
    pub fn new(
        config: crate::config::Config,
        instance: String,
        redis: redis::Client,
        nats: nats::jetstream::JetStream,
    ) -> Self {
        let rc_arc = std::sync::Arc::new(redis);
        let t_rc_arc = rc_arc.clone();
        let t_instance_id = instance.clone();
        let t2_instance_id = instance.clone();
        let sess_arc = std::sync::Arc::new(std::sync::RwLock::new(HashMap::<String, Socket>::new()));
        let t_sess_arc = sess_arc.clone();
        let t2_sess_arc = sess_arc.clone();

        match config.server.stats_interval_sec {
            | Some(v) => {
                let stats_interval: u64 = v.into();
                std::thread::spawn(move || loop {
                    crate::logger::LogMessage::now(&t2_instance_id, crate::logger::Data::Interval {
                        stats: crate::logger::Stats::Connections {
                            count: t2_sess_arc.read().unwrap().len(),
                            connections: t2_sess_arc.read().unwrap().keys().into_iter().collect(),
                        },
                    });
                    std::thread::sleep(std::time::Duration::from_secs(stats_interval));
                });
            },
            | None => {},
        }

        std::thread::spawn(move || {
            let mut conn = t_rc_arc.get_connection().unwrap();
            let mut ps = conn.as_pubsub();
            ps.subscribe(&t_instance_id).unwrap();

            loop {
                let mut safecall = || -> Result<(), Box<dyn std::error::Error>> {
                    let msg = ps.get_message()?;
                    let pl: String = msg.get_payload()?;
                    let payload: spoderman_bus::redis::Message = serde_json::from_str(&pl)?;
                    match payload {
                        | spoderman_bus::redis::Message::S2CMessage {
                            connection,
                            time: _,
                            message,
                        } => {
                            crate::logger::LogMessage::now(&t_instance_id, crate::logger::Data::Event {
                                data: crate::logger::Event::ServerMessagePost {
                                    connection: &connection,
                                },
                            });
                            match t_sess_arc.read()?.get(&connection) {
                                | Some(s) => {
                                    s.do_send(crate::messages::WsMessage::Message(message));
                                    Ok(())
                                },
                                | None => Err(Box::new(crate::error::ConnectionNotFoundError::new(
                                    &connection.to_string(),
                                ))),
                            }
                        },
                        | spoderman_bus::redis::Message::SDisconnect {
                            connection,
                            time: _,
                            reason,
                        } => {
                            crate::logger::LogMessage::now(&t_instance_id, crate::logger::Data::Event {
                                data: crate::logger::Event::ServerDisconnect {
                                    connection: &connection,
                                    reason: &reason,
                                },
                            });
                            match t_sess_arc.read()?.get(&connection) {
                                | Some(s) => {
                                    s.do_send(crate::messages::WsMessage::Disconnect(reason));
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
                        crate::logger::LogMessage::now(&t_instance_id, crate::logger::Data::Event {
                            data: crate::logger::Event::Error { err: &e.to_string() },
                        });
                        std::thread::sleep(std::time::Duration::from_secs(5));
                    },
                }
            }
        });
        Server {
            config,
            instance,
            sessions: sess_arc,
            redis: rc_arc,
            nats: std::sync::Arc::new(nats),
        }
    }

    pub fn make_key(&self, connection: &str) -> String {
        format!("i2c:{}:{}", self.instance, connection.to_string())
    }

    pub fn make_reverse_key(&self, connection: &str) -> String {
        format!("c2i:{}", connection)
    }
}

impl Actor for Server {
    type Context = Context<Self>;
}

impl Handler<Connect> for Server {
    type Result = std::result::Result<(), u16>;

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
                .insert(msg.connection.clone(), msg.addr.clone()); // must never be poisoned
            match &self.config.routes.connect {
                | Some(c) => {
                    let mut req = ureq::post(&c.endpoint);
                    for (k, v) in c.headers.iter() {
                        req = req.set(k, v);
                    }

                    let resp = req.send_string(&serde_json::to_string(&crate::routes::ConnectRequest {
                        instance_id: self.instance.clone(),
                        connection_id: msg.connection.clone(),
                        time: msg.time.clone(),
                    })?)?;

                    crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                        data: crate::logger::Event::ConnectRouteResponse {
                            connection: &msg.connection,
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
                },
                | None => Ok(()),
            }?;

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

impl Handler<Disconnect> for Server {
    type Result = std::result::Result<(), u16>;

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
                | Some(c) => {
                    let mut req = ureq::post(&c.endpoint);
                    for (k, v) in c.headers.iter() {
                        req = req.set(k, v);
                    }

                    let resp = req.send_string(&serde_json::to_string(&crate::routes::DisconnectRequest {
                        instance_id: self.instance.clone(),
                        connection_id: msg.connection.clone(),
                        time: msg.time.clone(),
                    })?)?;

                    crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                        data: crate::logger::Event::DisconnectRouteResponse {
                            connection: &msg.connection.to_string(),
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
                },
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

impl Handler<Heartbeat> for Server {
    type Result = std::result::Result<(), u16>;

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

impl Handler<ServerMessage> for Server {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let safecall = || -> Result<(), Box<dyn std::error::Error>> {
            crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                data: crate::logger::Event::ServerMessageEnqueue {
                    connection: &msg.connection.to_string(),
                },
            });

            let conn = msg.connection.clone();
            let redis_message: spoderman_bus::redis::Message = msg.into();

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

impl Handler<ServerDisconnect> for Server {
    type Result = ();

    fn handle(&mut self, msg: ServerDisconnect, _ctx: &mut Context<Self>) -> Self::Result {
        let safecall = || -> Result<(), Box<dyn std::error::Error>> {
            crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                data: crate::logger::Event::ServerDisconnect {
                    connection: &msg.connection.to_string(),
                    reason: &msg.reason,
                },
            });

            let conn = msg.connection.clone();
            let redis_message: spoderman_bus::redis::Message = msg.into();

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

impl Handler<ClientMessage> for Server {
    type Result = std::result::Result<(), u16>;

    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let safecall = || -> Result<(), Box<dyn std::error::Error>> {
            crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                data: crate::logger::Event::ClientMessage {
                    connection: &msg.connection.to_string(),
                },
            });

            self.nats.publish(
                &self.config.nats.stream,
                serde_json::json!(spoderman_bus::nats::ClientMessage {
                    instance_id: self.instance.clone(),
                    connection_id: msg.connection,
                    context: msg.context.into(),
                    time: msg.time,
                    message: msg.message,
                })
                .to_string(),
            )?;
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
