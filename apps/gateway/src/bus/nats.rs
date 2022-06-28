#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientMessage {
    pub instance_id: String,
    pub connection_id: String,
    pub context: crate::messages::MessageContext,
    pub time: String,
    pub message: String,
}
