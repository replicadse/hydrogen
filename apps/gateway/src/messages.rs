use actix::prelude::{
    Message,
    Recipient,
};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[derive(Message)]
#[rtype(result = "std::result::Result<(), u16>")]
pub struct Connect {
    pub connection: Uuid,
    pub addr: Recipient<WsMessage>,
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), u16>")]
pub struct Disconnect {
    pub connection: Uuid,
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), u16>")]
pub struct Heartbeat {
    pub connection: Uuid,
}

#[derive(Debug, Message, serde::Serialize, serde::Deserialize)]
#[rtype(result = "std::result::Result<(), u16>")]
pub struct ClientMessage {
    pub connection: Uuid,
    pub message: String,
}

#[derive(Debug, Message, serde::Serialize, serde::Deserialize)]
#[rtype(result = "()")]
pub struct ServerMessage {
    pub connection: Uuid,
    pub message: String,
}
