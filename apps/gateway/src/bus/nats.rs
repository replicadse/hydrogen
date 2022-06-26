#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ClientMessage<'a> {
    pub instance_id: &'a str,
    pub connection_id: &'a str,
    pub time: &'a str,
    pub message: &'a str,
}
