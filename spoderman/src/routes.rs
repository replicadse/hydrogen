#[derive(Debug, serde::Serialize)]
pub struct AuthorizerRequest<'a> {
    pub instance_id: &'a str,
    pub connection_id: &'a str,
    pub headers: Vec<(String, String)>,
}

#[derive(Debug, serde::Serialize)]
pub struct ConnectRequest<'a> {
    pub instance_id: String,
    pub connection_id: &'a str,
}

#[derive(Debug, serde::Serialize)]
pub struct RulesEngineRequest<'a> {
    pub instance_id: &'a str,
    pub connection_id: &'a str,
    pub msg: &'a str,
}

#[derive(Debug, serde::Deserialize)]
pub struct RulesEngineResponse {
    pub endpoint: String,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ForwardRequest<'a> {
    pub instance_id: &'a str,
    pub connection_id: &'a str,
    pub message: &'a str,
}

#[derive(Debug, serde::Serialize)]
pub struct DisconnectRequest<'a> {
    pub instance_id: &'a str,
    pub connection_id: &'a str,
}
