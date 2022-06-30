#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub version: String,
    pub engine_mode: EngineMode,
    pub queue: Queue,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Queue {
    Nats { endpoint: String, stream: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineMode {
    Regex {
        rules: std::vec::Vec<RegexRule>,
    },
    Dss {
        rules_engine: RulesEngineRoute,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RulesEngineRoute {
    pub endpoint: String,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DestinationRoute {
    pub endpoint: String,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegexRule {
    pub regex: String,
    pub route: DestinationRoute,
}
