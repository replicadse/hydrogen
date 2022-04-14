#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub version: String,
    pub server: Server,
    pub redis: Redis,
    pub routes: Routes,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Routes {
    pub authorizer: std::option::Option<Authorizer>,

    pub connect: std::option::Option<ConnectRoute>,
    pub disconnect: std::option::Option<DisconnectRoute>,

    pub rules_engine: RulesEngine,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Server {
    pub address: String,
    pub heartbeat_interval_sec: u16,
    pub stats_interval_sec: u16,
    pub connection_timeout_sec: u16,
    pub max_out_message_size: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Authorizer {
    pub endpoint: String,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Redis {
    pub endpoint: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectRoute {
    pub endpoint: String,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DisconnectRoute {
    pub endpoint: String,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RulesEngine {
    pub endpoint: String,
    pub headers: std::collections::HashMap<String, String>,
}
