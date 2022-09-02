#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Message {
    S2CMessage {
        connection: String,
        time: String,
        message: String,
    },
    SDisconnect {
        connection: String,
        time: String,
        reason: String,
    },
}
