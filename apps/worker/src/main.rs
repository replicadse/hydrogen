mod args;
mod config;
mod error;
mod logger;
mod routes;

use std::error::Error;

use futures::StreamExt;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    logger::LogMessage::now("-", logger::Data::Event {
        data: logger::Event::Startup { message: "startup" },
    });
    let instance = uuid::Uuid::new_v4();
    let args = args::ClapArgumentLoader::load()?;
    match args.command {
        | args::Command::Work { config } => {
            match &config.queue {
                | config::Queue::Nats {
                    endpoint,
                    stream: topic,
                } => {
                    endless_nats_consumer(&instance.to_string(), &config, &endpoint, &topic).await?;
                },
            }
            Ok(())
        },
    }
}

async fn endless_nats_consumer(
    instance: &str,
    config: &crate::config::Config,
    nats_endpoint: &str,
    stream: &str,
) -> std::result::Result<(), Box<dyn Error>> {
    let nc = async_nats::connect(nats_endpoint).await?;
    let nc2 = async_nats::jetstream::new(nc);
    let stream = nc2
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: stream.to_owned(),
            max_messages: 4096,
            max_messages_per_subject: 1024,
            discard: async_nats::jetstream::stream::DiscardPolicy::Old,
            retention: async_nats::jetstream::stream::RetentionPolicy::WorkQueue,
            max_message_size: 1024 * 256,
            ..Default::default()
        })
        .await
        .unwrap();
    let consumer = stream
        .get_or_create_consumer("worker", async_nats::jetstream::consumer::pull::Config {
            durable_name: Some("worker".to_owned()),
            deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::All,
            max_deliver: 1,
            max_ack_pending: 256,
            ..Default::default()
        })
        .await
        .unwrap();

    let mut messages = consumer.stream().unwrap();
    while let Some(Ok(message)) = messages.next().await {
        let msg_str = String::from_utf8(message.payload.to_vec())?;
        let msg_typed: spoderman_bus::nats::ClientMessage = serde_json::from_str(&msg_str)?;
        match handle_nats_message(instance, config, &msg_typed) {
            | Ok(..) => {
                message.ack().await.unwrap();
            },
            | Err(e) => crate::logger::LogMessage::now(instance, crate::logger::Data::Event {
                data: crate::logger::Event::Error {
                    message: &format!("error on message: {:?}, details: {}", msg_typed, e.to_string()),
                },
            }),
        }
    }
    Ok(())
}

trait RegexFindFirstMatching {
    fn first_regex_matches(&self, msg: &spoderman_bus::nats::ClientMessage) -> std::result::Result<std::option::Option<&crate::config::DestinationRoute>, Box<dyn std::error::Error>>;
}

impl RegexFindFirstMatching for std::vec::Vec<crate::config::RegexRule> {
    fn first_regex_matches(&self, msg: &spoderman_bus::nats::ClientMessage) -> std::result::Result<std::option::Option<&crate::config::DestinationRoute>, Box<dyn std::error::Error>> {
        for rule in self {
            let regex = match fancy_regex::Regex::new(&rule.regex) {
                Ok(it) => {it},
                Err(err) => return Err(Box::new(crate::error::InvalidRegexError::new(&err.to_string()))),
            };
            if regex.is_match(&msg.message)? {
                return Ok(Some(&rule.route));
            }
        }
        Ok(None)
    }
}

fn handle_nats_message(
    instance: &str,
    config: &crate::config::Config,
    msg: &spoderman_bus::nats::ClientMessage,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    crate::logger::LogMessage::now(instance, crate::logger::Data::Event {
        data: crate::logger::Event::Message {
            connection: &msg.connection_id.to_string(),
        },
    });

    match &config.engine_mode {
        config::EngineMode::Dss { rules_engine } => {
            handle_nats_message_dss_mode(instance, msg, &rules_engine)
        },
        config::EngineMode::Regex { rules } => {
            let dest = rules.first_regex_matches(msg)?;
            match dest {
                Some(v) => {
                    handle_nats_message_regex_mode(instance, msg, v)
                },
                None => {
                    crate::logger::LogMessage::now(instance, crate::logger::Data::Event {
                        data: crate::logger::Event::DroppedMessageNoMatch {
                            connection: &msg.connection_id.to_string(),
                        },
                    });
                    Ok(())
                }
            }
        }
    }
}

fn handle_nats_message_regex_mode(
    instance: &str,
    msg: &spoderman_bus::nats::ClientMessage,
    destination_route: &crate::config::DestinationRoute,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut destination_req = ureq::post(&destination_route.endpoint);
    for h in destination_route.headers.iter() {
        destination_req = destination_req.set(h.0, h.1);
    }

    let forward_resp = destination_req.send_string(&serde_json::to_string(&crate::routes::ForwardRequest {
        instance_id: msg.instance_id.clone(),
        connection_id: msg.connection_id.clone(),
        time: msg.time.clone(),
        context: crate::routes::MessageContext {
            authorizer: msg.context.authorizer.clone(),
        },
        message: msg.message.clone(),
    })?)?;

    crate::logger::LogMessage::now(instance, crate::logger::Data::Event {
        data: crate::logger::Event::DestinationRouteResponse {
            connection: &msg.connection_id.to_string(),
            response: forward_resp.status(),
        },
    });

    match forward_resp.status() {
        | 200 => Ok(()),
        | _ => Err(Box::new(crate::error::ForwardRouteError::new(&format!(
            "forward route error code {}",
            forward_resp.status()
        )))),
    }
}

fn handle_nats_message_dss_mode(
    instance: &str,
    msg: &spoderman_bus::nats::ClientMessage,
    rules_engine_route: &crate::config::RulesEngineRoute,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut re_req = ureq::post(&rules_engine_route.endpoint);
    for (k, v) in rules_engine_route.headers.iter() {
        re_req = re_req.set(k, v);
    }

    let re_response = re_req.send_string(&serde_json::to_string(&crate::routes::RulesEngineRequest {
        instance_id: msg.instance_id.clone(),
        connection_id: msg.connection_id.clone(),
        time: msg.time.clone(),
        context: crate::routes::MessageContext {
            authorizer: msg.context.authorizer.clone(),
        },
        message: msg.message.clone(),
    })?)?;

    crate::logger::LogMessage::now(instance, crate::logger::Data::Event {
        data: crate::logger::Event::RulesEngineRouteResponse {
            connection: &msg.connection_id.to_string(),
            response: re_response.status(),
        },
    });

    if re_response.status() != 200 {
        return Err(Box::new(crate::error::RulesEngineRouteError::new(&format!(
            "rules engine route error code {}",
            re_response.status()
        ))));
    }
    let re_response_parsed = serde_json::from_str::<crate::routes::RulesEngineResponse>(&re_response.into_string()?)?;

    let mut destination_req = ureq::post(&re_response_parsed.endpoint);
    for h in re_response_parsed.headers.iter() {
        destination_req = destination_req.set(h.0, h.1);
    }

    let forward_resp = destination_req.send_string(&serde_json::to_string(&crate::routes::ForwardRequest {
        instance_id: msg.instance_id.clone(),
        connection_id: msg.connection_id.clone(),
        time: msg.time.clone(),
        context: crate::routes::MessageContext {
            authorizer: msg.context.authorizer.clone(),
        },
        message: msg.message.clone(),
    })?)?;

    crate::logger::LogMessage::now(instance, crate::logger::Data::Event {
        data: crate::logger::Event::DestinationRouteResponse {
            connection: &msg.connection_id.to_string(),
            response: forward_resp.status(),
        },
    });

    match forward_resp.status() {
        | 200 => Ok(()),
        | _ => Err(Box::new(crate::error::ForwardRouteError::new(&format!(
            "forward route error code {}",
            forward_resp.status()
        )))),
    }
}
