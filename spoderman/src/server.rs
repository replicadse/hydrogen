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
    ServerMessage,
};

type Socket = actix::prelude::Recipient<crate::messages::WsMessage>;

pub struct Server {
    config: crate::config::Config,
    instance: uuid::Uuid,
    sessions: std::sync::Arc<std::sync::RwLock<HashMap<uuid::Uuid, Socket>>>,
    redis: std::sync::Arc<redis::Client>,
}

impl Server {
    pub fn new(config: crate::config::Config, instance: uuid::Uuid, redis: redis::Client) -> Self {
        let rc_arc = std::sync::Arc::new(redis);
        let t_rc_arc = rc_arc.clone();
        let t_instance_id = instance.to_string();
        let t2_instance_id = instance.to_string();
        let sess_arc = std::sync::Arc::new(std::sync::RwLock::new(HashMap::<uuid::Uuid, Socket>::new()));
        let t_sess_arc = sess_arc.clone();
        let t2_sess_arc = sess_arc.clone();
        let stats_interval: u64 = config.server.stats.into();

        std::thread::spawn(move || loop {
            crate::logger::LogMessage::now(&t2_instance_id, crate::logger::Data::Interval {
                stats: crate::logger::Stats::ConnectedClients {
                    count: t2_sess_arc.read().unwrap().len(),
                    clients: t2_sess_arc.read().unwrap().keys().into_iter().collect(),
                },
            });
            std::thread::sleep(std::time::Duration::from_secs(stats_interval));
        });

        std::thread::spawn(move || {
            let mut conn = t_rc_arc.get_connection().unwrap();
            let mut ps = conn.as_pubsub();
            ps.subscribe(&t_instance_id).unwrap();

            loop {
                let mut safecall = || -> Result<(), Box<dyn std::error::Error>> {
                    let msg = ps.get_message()?;
                    let pl: String = msg.get_payload()?;
                    let payload: ServerMessage = serde_json::from_str::<crate::messages::ServerMessage>(&pl)?;
                    crate::logger::LogMessage::now(&t_instance_id, crate::logger::Data::Event {
                        data: crate::logger::Event::ServerMessagePost {
                            connection: &payload.connection.to_string(),
                        },
                    });
                    match t_sess_arc.read()?.get(&payload.connection) {
                        | Some(s) => {
                            s.do_send(crate::messages::WsMessage(payload.msg))?;
                            Ok(())
                        },
                        | None => Err(Box::new(crate::error::ConnectionNotFoundError::new(
                            &payload.connection.to_string(),
                        ))),
                    }
                };
                match safecall() {
                    | Ok(_) => {},
                    | Err(e) => {
                        crate::logger::LogMessage::now(&instance.to_string(), crate::logger::Data::Event {
                            data: crate::logger::Event::Error { err: &e.to_string() },
                        });
                    },
                }
            }
        });
        Server {
            config,
            instance,
            sessions: sess_arc,
            redis: rc_arc,
        }
    }

    pub fn make_key(&self, connection: uuid::Uuid) -> String {
        format!("i2c:{}:{}", self.instance.to_string(), connection.to_string())
    }

    pub fn make_reverse_key(&self, connection: uuid::Uuid) -> String {
        format!("c2i:{}", connection.to_string())
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
                        instance_id: self.instance.to_string(),
                        connection_id: &msg.connection.to_string(),
                    })?)?;

                    crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                        data: crate::logger::Event::ConnectRouteResponse {
                            connection: &msg.connection.to_string(),
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

            let key = self.make_key(msg.connection);
            let rkey = self.make_reverse_key(msg.connection);

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

            let key = self.make_key(msg.connection);
            let rkey = self.make_reverse_key(msg.connection);
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
                        instance_id: &self.instance.to_string(),
                        connection_id: &msg.connection.to_string(),
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
            let key = self.make_key(msg.connection);
            let rkey = self.make_reverse_key(msg.connection);
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

            let target_instance = redis::cmd("GET")
                .arg(&self.make_reverse_key(msg.connection))
                .query::<String>(&mut self.redis.get_connection()?)?;
            redis::pipe()
                .publish(target_instance, serde_json::to_string(&msg)?)
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

            let mut re_req = ureq::post(&self.config.routes.rules_engine.endpoint);
            for (k, v) in self.config.routes.rules_engine.headers.iter() {
                re_req = re_req.set(k, v);
            }

            let re_response = re_req.send_string(&serde_json::to_string(&crate::routes::RulesEngineRequest {
                instance_id: &self.instance.to_string(),
                connection_id: &msg.connection.to_string(),
                msg: &msg.msg,
            })?)?;

            crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                data: crate::logger::Event::RulesEngineRouteResponse {
                    connection: &msg.connection.to_string(),
                    response: re_response.status(),
                },
            });

            if re_response.status() != 200 {
                return Err(Box::new(crate::error::RulesEngineRouteError::new(&format!(
                    "rules engine route error code {}",
                    re_response.status()
                ))));
            }
            let re_response_parsed =
                serde_json::from_str::<crate::routes::RulesEngineResponse>(&re_response.into_string()?)?;

            let mut forwerd_req = ureq::post(&re_response_parsed.endpoint);
            for h in re_response_parsed.headers.iter() {
                forwerd_req = forwerd_req.set(&h[0], &h[1]);
            }

            let forward_resp = forwerd_req.send_string(&serde_json::to_string(&crate::routes::ForwardRequest {
                instance_id: &self.instance.to_string(),
                connection_id: &msg.connection.to_string(),
                message: &msg.msg,
            })?)?;

            crate::logger::LogMessage::now(&self.instance.to_string(), crate::logger::Data::Event {
                data: crate::logger::Event::ForwardRouteResponse {
                    connection: &msg.connection.to_string(),
                    response: forward_resp.status(),
                },
            });

            match forward_resp.status() {
                | 200 => Ok(()),
                | _ => Err(Box::new(crate::error::ForwardRouteError::new(&format!(
                    "forward route error code {}",
                    forward_resp.status()
                )))),
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
