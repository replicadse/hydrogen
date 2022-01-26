use actix::prelude::{
    Message,
    Recipient,
};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub connection: Uuid,
    pub addr: Recipient<WsMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub connection: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Heartbeat {
    pub connection: Uuid,
}

#[derive(Debug, Message, serde::Serialize, serde::Deserialize)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub connection: Uuid,
    pub msg: String,
}

#[derive(Debug, Message, serde::Serialize, serde::Deserialize)]
#[rtype(result = "()")]
pub struct ServerMessage {
    pub connection: Uuid,
    pub msg: String,
}
