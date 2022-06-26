#[derive(serde::Serialize, serde::Deserialize)]
pub struct ClientMessage {
    instance_id: String,
    connection_id: String,
    message: String,
}
