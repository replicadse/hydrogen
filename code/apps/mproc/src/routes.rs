pub type MessageContextMap = std::collections::HashMap<String, serde_json::Value>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MessageContext {
    pub authorizer: std::option::Option<MessageContextMap>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RulesEngineRequest {
    pub instance_id: String,
    pub connection_id: String,
    pub endpoint: String,
    pub time: String,
    pub context: MessageContext,
    pub message: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RulesEngineResponse {
    pub endpoint: String,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ForwardRequest {
    pub instance_id: String,
    pub connection_id: String,
    pub endpoint: String,
    pub time: String,
    pub context: MessageContext,
    pub message: String,
}
