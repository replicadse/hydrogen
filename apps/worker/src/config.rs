#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub version: String,
    pub queue: Queue,
    pub routes: Routes,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Queue {
    Nats {
        connection: String,
        stream_name: String,
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Routes {
    pub rules_engine: RulesEngineRoute,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RulesEngineRoute {
    pub endpoint: String,
    pub headers: std::collections::HashMap<String, String>,
}
