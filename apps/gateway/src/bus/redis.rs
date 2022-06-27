#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Message {
    Message {
        connection: String,
        time: String,
        message: String,
    },
    Disconnect {
        connection: String,
        time: String,
        reason: String,
    },
}
