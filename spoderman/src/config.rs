#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub version: String,
    pub server: Server,
    pub authorizer: std::option::Option<Authorizer>,
    pub redis: Redis,
    pub routes: Routes,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Routes {
    pub connect: std::option::Option<ConnectRoute>,
    pub disconnect: std::option::Option<DisconnectRoute>,

    pub rules_engine: RuleEngine,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Server {
    pub address: String,
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
pub struct RuleEngine {
    pub endpoint: String,
    pub headers: std::collections::HashMap<String, String>,
}
