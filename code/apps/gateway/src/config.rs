#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub version: String,
    pub group_id: String,
    pub server: Server,
    pub redis: Redis,
    pub nats: Nats,
    pub routes: Routes,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Routes {
    pub authorizer: std::option::Option<Authorizer>,
    pub connect: std::option::Option<ConnectRoute>,
    pub disconnect: std::option::Option<DisconnectRoute>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Server {
    pub address: String,
    pub heartbeat_interval_sec: u16,
    pub stats_interval_sec: std::option::Option<u16>,
    pub connection_timeout_sec: u16,
    pub max_out_message_size: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Authorizer {
    pub endpoint: String,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Redis {
    pub endpoint: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Nats {
    pub endpoint: String,
    pub stream: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ConnectRoute {
    pub endpoint: String,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DisconnectRoute {
    pub endpoint: String,
    pub headers: std::collections::HashMap<String, String>,
}
