use std::time::Instant;

use actix::{
    fut,
    Actor,
    ActorContext,
    ActorFutureExt,
    Addr,
    AsyncContext,
    ContextFutureSpawner,
    Handler,
    MailboxError,
    Running,
    StreamHandler,
    WrapFuture,
};
use actix_web_actors::{
    ws,
    ws::{
        CloseReason,
        Message::Text,
        WebsocketContext,
    },
};
use uuid::Uuid;

use crate::{
    messages::{
        ClientMessage,
        Connect,
        Disconnect,
        Heartbeat,
        WsMessage,
    },
    server::Server,
};

pub type WsConnContextMap = std::collections::HashMap<String, serde_json::Value>;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WsConnContext {
    pub authorizer: std::option::Option<WsConnContextMap>,
}

pub struct WsConn {
    address: Addr<Server>,
    heartbeat: Instant,
    pub connection: String,
    pub context: WsConnContext,
    heartbeat_int: std::time::Duration,
    timeout: std::time::Duration,
}

impl WsConn {
    pub fn claim_id() -> String {
        Uuid::new_v4().to_string()
    }

    pub fn new(
        connection: String,
        server: Addr<Server>,
        context: WsConnContext,
        heartbeat_int: std::time::Duration,
        timeout: std::time::Duration,
    ) -> WsConn {
        WsConn {
            connection,
            address: server,
            heartbeat: Instant::now(),
            context,
            heartbeat_int,
            timeout,
        }
    }
}

impl Actor for WsConn {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx, self.heartbeat_int, self.timeout);

        let addr = ctx.address();
        self.address
            .send(Connect {
                addr: addr.recipient(),
                time: chrono::Utc::now().to_rfc3339(),
                connection: self.connection.clone(),
            })
            .into_actor(self)
            .then(
                |res: std::result::Result<std::result::Result<(), u16>, MailboxError>,
                 _,
                 ctx: &mut WebsocketContext<WsConn>| {
                    match res {
                        | Ok(inner) => match inner {
                            | Ok(..) => (),
                            | Err(..) => ctx.stop(),
                        },
                        | Err(..) => ctx.stop(),
                    }
                    fut::ready(())
                },
            )
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.address.do_send(Disconnect {
            connection: self.connection.clone(),
            time: chrono::Utc::now().to_rfc3339(),
        });
        Running::Stop
    }
}

impl WsConn {
    fn heartbeat(
        &self,
        ctx: &mut ws::WebsocketContext<Self>,
        interval: std::time::Duration,
        timeout: std::time::Duration,
    ) {
        ctx.run_interval(interval, move |act, ctx| {
            if Instant::now().duration_since(act.heartbeat) > timeout {
                act.address.do_send(Disconnect {
                    connection: act.connection.clone(),
                    time: chrono::Utc::now().to_rfc3339(),
                });
                ctx.stop();
                return;
            }
            ctx.ping(b".");
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConn {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            | Ok(ws::Message::Ping(msg)) => {
                self.heartbeat = Instant::now();
                self.address.do_send(Heartbeat {
                    connection: self.connection.clone(),
                    time: chrono::Utc::now().to_rfc3339(),
                });
                ctx.pong(&msg);
            },
            | Ok(ws::Message::Pong(_)) => {
                self.heartbeat = Instant::now();
                self.address.do_send(Heartbeat {
                    connection: self.connection.clone(),
                    time: chrono::Utc::now().to_rfc3339(),
                });
            },
            | Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            | Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            },
            | Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            },
            | Ok(ws::Message::Nop) => (),
            | Ok(Text(s)) => self.address.do_send(ClientMessage {
                connection: self.connection.clone(),
                time: chrono::Utc::now().to_rfc3339(),
                context: crate::messages::ConnectionContext {
                    authorizer: self.context.authorizer.clone(),
                },
                message: s.to_string(),
            }),
            | Err(e) => panic!("{}", e),
        }
    }
}

impl Handler<WsMessage> for WsConn {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        match msg {
            | WsMessage::Message(v) => {
                ctx.text(v);
            },
            | WsMessage::Disconnect(v) => {
                ctx.close(Some(CloseReason {
                    code: ws::CloseCode::Policy,
                    description: Some(v),
                }));
                ctx.stop();
            },
        }
    }
}
