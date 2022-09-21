#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthorizerRequest {
    pub instance_id: String,
    pub endpoint: String,
    pub connection_id: String,
    pub time: String,
    pub headers: Vec<(String, String)>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthorizerResponse {
    pub context: std::option::Option<crate::ws::WsConnContextMap>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ConnectRequest {
    pub instance_id: String,
    pub endpoint: String,
    pub connection_id: String,
    pub time: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DisconnectRequest {
    pub instance_id: String,
    pub endpoint: String,
    pub connection_id: String,
    pub time: String,
}
