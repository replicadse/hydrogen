#[derive(Debug, serde::Serialize)]
pub struct AuthorizerRequest {
    pub instance_id: String,
    pub connection_id: String,
    pub time: String,
    pub headers: Vec<(String, String)>,
}

#[derive(Debug, serde::Deserialize)]
pub struct AuthorizerResponse {
    pub context: std::option::Option<crate::ws::WsConnContextMap>,
}

#[derive(Debug, serde::Serialize)]
pub struct ConnectRequest {
    pub instance_id: String,
    pub connection_id: String,
    pub time: String,
}

#[derive(Debug, serde::Serialize)]
pub struct DisconnectRequest {
    pub instance_id: String,
    pub connection_id: String,
    pub time: String,
}
