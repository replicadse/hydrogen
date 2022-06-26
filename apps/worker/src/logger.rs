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
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Event<'a> {
    Startup { message: &'a str },
}

