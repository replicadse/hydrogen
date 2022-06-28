pub type MessageContextMap = std::collections::HashMap<String, serde_json::Value>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectionContext {
    pub authorizer: std::option::Option<MessageContextMap>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientMessage {
    pub instance_id: String,
    pub connection_id: String,
    pub context: ConnectionContext,
    pub time: String,
    pub message: String,
}
