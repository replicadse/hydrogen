#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Message {
    ServerMessage {
        connection: String,
        time: String,
        message: String,
    },
    ServerDisconnect {
        connection: String,
        time: String,
        reason: String,
    },
}
