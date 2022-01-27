#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct LogMessage {
    time: String,
    instance: String,
    data: Data,
}

impl LogMessage {
    pub fn now(instance: String, data: Data) -> () {
        Self {
            time: chrono::Utc::now().to_rfc3339(),
            instance,
            data,
        }
        .log()
    }

    pub fn log(&self) -> () {
        let msg = serde_json::to_string(self)
            .or::<String>(Ok("can not serialize log message".to_owned()))
            .unwrap();
        println!("{}", msg);
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Data {
    Event { data: Event },
    Interval { stats: Stats },
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Error { err: String },
    Startup { message: String },

    Connect { connection: String },
    Disconnect { connection: String },

    ClientMessage { connection: String },
    ServerMessageEnqueue { connection: String },
    ServerMessagePost { connection: String },

    AuthRouteResponse { connection: String, response: u16 },
    ConnectRouteResponse { connection: String, response: u16 },
    RulesEngineRouteResponse { connection: String, response: u16 },
    ForwardRouteResponse { connection: String, response: u16 },
    DisconnectRouteResponse { connection: String, response: u16 },
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stats {
    ConnectedClients { count: usize },
}
