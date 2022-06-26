mod args;
mod config;
mod error;
mod logger;
mod routes;
mod bus {
    pub mod nats;
}

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
                    nats(&instance.to_string(), &config, &endpoint, &topic).await?;
                },
            }
            Ok(())
        },
    }
}

async fn nats(
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
        println!("{:?}", message);
        let msg_str = String::from_utf8(message.payload.to_vec())?;
        let msg_typed: crate::bus::nats::ClientMessage = serde_json::from_str(&msg_str)?;
        match handle_message(instance, config, &msg_typed) {
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

fn handle_message(
    instance: &str,
    config: &crate::config::Config,
    msg: &crate::bus::nats::ClientMessage,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut re_req = ureq::post(&config.routes.rules_engine.endpoint);
    for (k, v) in config.routes.rules_engine.headers.iter() {
        re_req = re_req.set(k, v);
    }

    let re_response = re_req.send_string(&serde_json::to_string(&crate::routes::RulesEngineRequest {
        instance_id: &msg.instance_id.to_string(),
        connection_id: &msg.connection_id.to_string(),
        time: msg.time,
        message: &msg.message,
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

    let mut forwerd_req = ureq::post(&re_response_parsed.endpoint);
    for h in re_response_parsed.headers.iter() {
        forwerd_req = forwerd_req.set(h.0, h.1);
    }

    let forward_resp = forwerd_req.send_string(&serde_json::to_string(&crate::routes::ForwardRequest {
        instance_id: &msg.instance_id.to_string(),
        connection_id: &msg.connection_id.to_string(),
        time: msg.time,
        message: &msg.message,
    })?)?;

    crate::logger::LogMessage::now(instance, crate::logger::Data::Event {
        data: crate::logger::Event::ForwardRouteResponse {
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
