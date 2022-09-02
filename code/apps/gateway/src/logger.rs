#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct LogMessage<'a> {
    time: String,
    instance: &'a str,
    data: Data<'a>,
}

impl<'a> LogMessage<'a> {
    pub fn now(instance: &'a str, data: Data<'a>) -> () {
        Self {
            time: chrono::Utc::now().to_rfc3339(),
            instance,
            data,
        }
        .log()
    }

    pub fn log(&self) -> () {
        match serde_json::to_string(self) {
            | Ok(v) => println!("{}", v),
            | Err(e) => println!("{}", e),
        }
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Data<'a> {
    Event { data: Event<'a> },
    Interval { stats: Stats<'a> },
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Event<'a> {
    Error { err: &'a str },
    Startup { message: &'a str },

    Connect { connection: &'a str },
    Disconnect { connection: &'a str },
    ServerDisconnect { connection: &'a str, reason: &'a str },

    ClientMessage { connection: &'a str },
    ServerMessageEnqueue { connection: &'a str },
    ServerMessagePost { connection: &'a str },

    AuthRouteResponse { connection: &'a str, response: u16 },
    ConnectRouteResponse { connection: &'a str, response: u16 },
    DisconnectRouteResponse { connection: &'a str, response: u16 },
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stats<'a> {
    Connections { count: usize, connections: Vec<&'a String> },
}
