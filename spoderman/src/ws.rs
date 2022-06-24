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

pub struct WsConn {
    address: Addr<Server>,
    heartbeat: Instant,
    pub connection: Uuid,
    heartbeat_int: std::time::Duration,
    timeout: std::time::Duration,
}

impl WsConn {
    pub fn new(server: Addr<Server>, heartbeat_int: std::time::Duration, timeout: std::time::Duration) -> WsConn {
        WsConn {
            connection: Uuid::new_v4(),
            heartbeat: Instant::now(),
            address: server,
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
                connection: self.connection,
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
            connection: self.connection,
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
                    connection: act.connection,
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
                    connection: self.connection,
                });
                ctx.pong(&msg);
            },
            | Ok(ws::Message::Pong(_)) => {
                self.heartbeat = Instant::now();
                self.address.do_send(Heartbeat {
                    connection: self.connection,
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
                connection: self.connection,
                msg: s.to_string(),
            }),
            | Err(e) => panic!("{}", e),
        }
    }
}

impl Handler<WsMessage> for WsConn {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
