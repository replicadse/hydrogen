mod args;
mod config;
mod error;
mod logger;
mod messages;
mod routes;
mod send_handler;
mod server;
mod websocket_handler;
mod ws;

use std::error::Error;

use actix::Actor;
use actix_web::{
    web::Data,
    App,
    HttpServer,
};
use send_handler::handler as s_handler;
use server::Server;
use websocket_handler::handler as ws_handler;

#[actix_web::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    logger::LogMessage::now("-", logger::Data::Event {
        data: logger::Event::Startup { message: "startup" },
    });
    let args = args::ClapArgumentLoader::load()?;
    match args.command {
        | args::Command::Serve { config } => {
            serve(config).await?;
            Ok(())
        },
    }
}

async fn serve(config: crate::config::Config) -> std::result::Result<(), Box<dyn Error>> {
    let instance = uuid::Uuid::new_v4();
    logger::LogMessage::now(&instance.to_string(), logger::Data::Event {
        data: logger::Event::Startup {
            message: &format!("new instance {}", &instance),
        },
    });

    let redis = redis::Client::open(config.redis.endpoint.clone())?;
    logger::LogMessage::now(&instance.to_string(), logger::Data::Event {
        data: logger::Event::Startup {
            message: &format!("redis client opened @ {}", &config.redis.endpoint),
        },
    });
    if let Err(e) = redis.get_connection() {
        let e = &format!("could not connect to {}, {}", &config.redis.endpoint, e.to_string());
        logger::LogMessage::now(&instance.to_string(), logger::Data::Event {
            data: logger::Event::Error { err: e },
        });
        return Err(Box::new(error::StartupError::new(e)));
    }

    let bind = config.server.address.clone();
    logger::LogMessage::now(&instance.to_string(), logger::Data::Event {
        data: logger::Event::Startup {
            message: &format!("instance will bind @ {}", &bind),
        },
    });

    let server = Server::new(config.clone(), instance, redis.clone()).start();
    HttpServer::new(move || {
        App::new()
            .service(ws_handler)
            .app_data(Data::new(server.clone()))
            .app_data(Data::new(instance.clone()))
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(redis.clone()))
            .service(s_handler)
            .app_data(Data::new(server.clone()))
            .app_data(Data::new(config.clone()))
    })
    .bind(&bind)?
    // .disable_signals()
    .run()
    .await?;
    Ok(())
}
