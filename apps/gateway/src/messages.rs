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

#[derive(Debug, Message, serde::Serialize, serde::Deserialize)]
#[rtype(result = "std::result::Result<(), u16>")]
pub struct ClientMessage {
    pub connection: String,
    pub time: String,
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

impl Into<crate::bus::redis::Message> for ServerMessage {
    fn into(self) -> crate::bus::redis::Message {
        crate::bus::redis::Message::Message {
            connection: self.connection.to_string(),
            time: self.time,
            message: self.message,
        }
    }
}

impl Into<crate::bus::redis::Message> for ServerDisconnect {
    fn into(self) -> crate::bus::redis::Message {
        crate::bus::redis::Message::Disconnect {
            connection: self.connection.to_string(),
            time: self.time,
            reason: self.reason,
        }
    }
}
