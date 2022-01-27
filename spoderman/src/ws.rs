use std::time::{
    Duration,
    Instant,
};

use actix::{
    fut,
    Actor,
    ActorContext,
    ActorFuture,
    Addr,
    AsyncContext,
    ContextFutureSpawner,
    Handler,
    Running,
    StreamHandler,
    WrapFuture, MailboxError,
};
use actix_web_actors::{
    ws,
    ws::{Message::Text, WebsocketContext},
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

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WsConn {
    address: Addr<Server>,
    heartbeat: Instant,
    pub connection: Uuid,
}

impl WsConn {
    pub fn new(server: Addr<Server>) -> WsConn {
        WsConn {
            connection: Uuid::new_v4(),
            heartbeat: Instant::now(),
            address: server,
        }
    }
}

impl Actor for WsConn {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);

        let addr = ctx.address();
        self.address
            .send(Connect {
                addr: addr.recipient(),
                connection: self.connection,
            })
            .into_actor(self)
            .then(|res: std::result::Result<std::result::Result<(), u16>, MailboxError>, _, ctx: &mut WebsocketContext<WsConn>| {
                match res {
                    | Ok(inner) => {
                        match inner {
                            | Ok(..) => (),
                            | Err(..) => ctx.stop(),
                        }
                    },
                    | Err(..) => ctx.stop(),
                }
                fut::ready(())
            })
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
    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.heartbeat) > CONNECTION_TIMEOUT {
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
                msg: s,
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
