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

/// Type representing all relevant information about an established websocket
/// connection between a client and the server.
pub struct WsConn {
    address: Addr<Server>,
    heartbeat: Instant,
    pub connection: String,
    pub group: String,
    pub endpoint: String,
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
        group: String,
        endpoint: String,
        server: Addr<Server>,
        context: WsConnContext,
        heartbeat_int: std::time::Duration,
        timeout: std::time::Duration,
    ) -> WsConn {
        WsConn {
            connection,
            group,
            endpoint,
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

    /// Handles the connection initiation for a client to server connection when
    /// it has already been established.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx, self.heartbeat_int, self.timeout);

        let addr = ctx.address();
        self.address
            .send(Connect {
                addr: addr.recipient(),
                time: chrono::Utc::now().to_rfc3339(),
                connection: self.connection.clone(),
                group_id: self.group.clone(),
                endpoint: self.endpoint.clone(),
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

    /// Handles a connection which is in the process of bein stopped.
    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.address.do_send(Disconnect {
            connection: self.connection.clone(),
            group_id: self.group.clone(),
            endpoint: self.endpoint.clone(),
            time: chrono::Utc::now().to_rfc3339(),
        });
        Running::Stop
    }
}

impl WsConn {
    /// Runs a new heartbeat on a steady interval / rate.
    fn heartbeat(
        &self,
        ctx: &mut ws::WebsocketContext<Self>,
        interval: std::time::Duration,
        timeout: std::time::Duration,
    ) {
        let endpoint = self.endpoint.clone();
        let group = self.group.clone();
        ctx.run_interval(interval, move |act, ctx| {
            if Instant::now().duration_since(act.heartbeat) > timeout {
                act.address.do_send(Disconnect {
                    connection: act.connection.clone(),
                    group_id: group.clone(),
                    endpoint: endpoint.clone(),
                    time: chrono::Utc::now().to_rfc3339(),
                });
                ctx.stop();
                return;
            }
            ctx.ping(b".");
        });
    }
}

/// Main handler for all immediate socket and context related operations on the
/// connection.
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConn {
    /// This function will handle all various events that can occurr in a
    /// websocket connection such as heartbeats, messages and binary data
    /// transfer.
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
                group_id: self.group.clone(),
                endpoint: self.endpoint.clone(),
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

/// Handler for server to client events.
impl Handler<WsMessage> for WsConn {
    type Result = ();

    /// Will handle low-level server events for a given connection.
    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        match msg {
            | WsMessage::Message { message, .. } => {
                ctx.text(message);
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
