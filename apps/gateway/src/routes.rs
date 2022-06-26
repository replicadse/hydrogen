#[derive(Debug, serde::Serialize)]
pub struct AuthorizerRequest<'a> {
    pub instance_id: &'a str,
    pub connection_id: &'a str,
    pub time: &'a str,
    pub headers: Vec<(String, String)>,
}

#[derive(Debug, serde::Serialize)]
pub struct ConnectRequest<'a> {
    pub instance_id: String,
    pub connection_id: &'a str,
    pub time: &'a str,
}

#[derive(Debug, serde::Serialize)]
pub struct DisconnectRequest<'a> {
    pub instance_id: &'a str,
    pub connection_id: &'a str,
    pub time: &'a str,
}
