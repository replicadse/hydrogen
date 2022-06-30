#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct LogMessage<'a> {
    time: String,
    worker_instance: &'a str,
    data: Data<'a>,
}

impl<'a> LogMessage<'a> {
    pub fn now(worker_instance: &'a str, data: Data<'a>) -> () {
        Self {
            time: chrono::Utc::now().to_rfc3339(),
            worker_instance,
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
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Event<'a> {
    Error { message: &'a str },
    Startup { message: &'a str },

    RulesEngineRouteResponse { connection: &'a str, response: u16 },
    DestinationRouteResponse { connection: &'a str, response: u16 },

    Message { connection: &'a str },
    DroppedMessageNoMatch { connection: &'a str },
}
