use actix::prelude::{
    Message,
    Recipient,
};

#[derive(Message)]
#[rtype(result = "()")]
pub enum WsMessage {
    Message(String),
    Disconnect(String),
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), u16>")]
pub struct Connect {
    pub connection: String,
    pub time: String,
    pub addr: Recipient<WsMessage>,
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), u16>")]
pub struct Disconnect {
    pub connection: String,
    pub time: String,
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), u16>")]
pub struct Heartbeat {
    pub connection: String,
    pub time: String,
}

pub type ConnectionContextMap = std::collections::HashMap<String, serde_json::Value>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectionContext {
    pub authorizer: std::option::Option<ConnectionContextMap>,
}

#[derive(Debug, Message, serde::Serialize, serde::Deserialize)]
#[rtype(result = "std::result::Result<(), u16>")]
pub struct ClientMessage {
    pub connection: String,
    pub time: String,
    pub context: ConnectionContext,
    pub message: String,
}

#[derive(Debug, Message, serde::Serialize, serde::Deserialize)]
#[rtype(result = "()")]
pub struct ServerMessage {
    pub connection: String,
    pub time: String,
    pub message: String,
}

#[derive(Debug, Message, serde::Serialize, serde::Deserialize)]
#[rtype(result = "()")]
pub struct ServerDisconnect {
    pub connection: String,
    pub time: String,
    pub reason: String,
}

impl Into<spoderman_bus::redis::Message> for ServerMessage {
    fn into(self) -> spoderman_bus::redis::Message {
        spoderman_bus::redis::Message::S2CMessage {
            connection: self.connection.to_string(),
            time: self.time,
            message: self.message,
        }
    }
}

impl Into<spoderman_bus::redis::Message> for ServerDisconnect {
    fn into(self) -> spoderman_bus::redis::Message {
        spoderman_bus::redis::Message::SDisconnect {
            connection: self.connection.to_string(),
            time: self.time,
            reason: self.reason,
        }
    }
}

impl Into<spoderman_bus::nats::ConnectionContext> for ConnectionContext {
    fn into(self) -> spoderman_bus::nats::ConnectionContext {
        spoderman_bus::nats::ConnectionContext {
            authorizer: self.authorizer,
        }
    }
}
