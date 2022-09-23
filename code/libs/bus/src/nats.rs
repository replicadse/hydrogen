pub type MessageContextMap = std::collections::HashMap<String, serde_json::Value>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ConnectionContext {
    pub authorizer: std::option::Option<MessageContextMap>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ClientMessage {
    pub instance_id: String,
    pub connection_id: String,
    pub endpoint: String,
    pub context: ConnectionContext,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Message<T> {
    pub meta: MessageMeta,
    pub data: T,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MessageMeta {
    pub id: String,
    pub timestamp: String,
}
