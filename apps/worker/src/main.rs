mod args;
mod config;
mod error;
mod logger;
mod messages;

use std::error::Error;

use futures::StreamExt;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    logger::LogMessage::now("-", logger::Data::Event {
        data: logger::Event::Startup { message: "startup" },
    });
    let args = args::ClapArgumentLoader::load()?;
    match args.command {
        | args::Command::Work { config, publish } => {
            match config.queue {
                config::Queue::Nats { connection, stream_name: topic } => {
                    nats(publish, &connection, &topic).await?;
                }
            }
            Ok(())
        },
    }
}

/// BLOCKING_MAX_THREADS = 16
async fn nats(publish: bool, connection: &str, stream_name: &str) -> std::result::Result<(), Box<dyn Error>> {
    let nc = async_nats::connect(connection).await?;
    let nc2 = async_nats::jetstream::new(nc);
    let stream = nc2.get_or_create_stream(async_nats::jetstream::stream::Config{
        name: stream_name.to_owned(),
        max_messages: 4096,
        max_messages_per_subject: 1024,
        discard: async_nats::jetstream::stream::DiscardPolicy::Old,
        retention: async_nats::jetstream::stream::RetentionPolicy::WorkQueue,
        max_message_size: 1024 * 256,
        ..Default::default()
    }).await.unwrap();
    let consumer = stream.get_or_create_consumer("worker", async_nats::jetstream::consumer::pull::Config {
        durable_name: Some("worker".to_owned()),
        deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::All,
        max_deliver: 1,
        max_ack_pending: 256,
        ..Default::default()
    }).await.unwrap();

    if publish {
        loop {
            nc2.publish("client".to_owned(), "MESSAGE 1".into()).await.unwrap();
        }
    } else {
        let mut messages = consumer.stream().unwrap();
        while let Some(Ok(message)) = messages.next().await {
            println!("{:?}", message);
            message.ack().await.unwrap();
        };
    }

    Ok(())
}
