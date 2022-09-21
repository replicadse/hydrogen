use actix::prelude::{
    Message,
    Recipient,
};

#[derive(Message)]
#[rtype(result = "()")]
pub enum WsMessage {
    Message { endpoint: String, message: String },
    Disconnect(String),
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), u16>")]
pub struct Connect {
    pub connection: String,
    pub endpoint: String,
    pub time: String,
    pub addr: Recipient<WsMessage>,
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), u16>")]
pub struct Disconnect {
    pub connection: String,
    pub endpoint: String,
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
pub struct BroadcastServerMessage {
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

impl Into<hydrogen_bus::redis::Message> for BroadcastServerMessage {
    fn into(self) -> hydrogen_bus::redis::Message {
        hydrogen_bus::redis::Message::SBroadcast {
            time: self.time,
            message: self.message,
        }
    }
}

impl Into<hydrogen_bus::redis::Message> for ServerMessage {
    fn into(self) -> hydrogen_bus::redis::Message {
        hydrogen_bus::redis::Message::S2CMessage {
            connection: self.connection.to_string(),
            time: self.time,
            message: self.message,
        }
    }
}

impl Into<hydrogen_bus::redis::Message> for ServerDisconnect {
    fn into(self) -> hydrogen_bus::redis::Message {
        hydrogen_bus::redis::Message::SDisconnect {
            connection: self.connection.to_string(),
            time: self.time,
            reason: self.reason,
        }
    }
}

impl Into<hydrogen_bus::nats::ConnectionContext> for ConnectionContext {
    fn into(self) -> hydrogen_bus::nats::ConnectionContext {
        hydrogen_bus::nats::ConnectionContext {
            authorizer: self.authorizer,
        }
    }
}
