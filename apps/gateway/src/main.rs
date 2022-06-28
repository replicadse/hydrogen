mod args;
mod config;
mod error;
mod logger;
mod messages;
mod routes;
mod server;
mod ws;
mod macros;
mod handlers {
    pub mod connection;
    pub mod health;
    pub mod websocket;
}
mod bus {
    pub mod nats;
    pub mod redis;
}

use std::error::Error;

use actix::Actor;
use actix_web::{
    web::Data,
    App,
    HttpServer,
};
use server::Server;

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
    let instance = uuid::Uuid::new_v4().to_string();
    logger::LogMessage::now(&instance.to_string(), logger::Data::Event {
        data: logger::Event::Startup {
            message: &format!("new instance {}", &instance),
        },
    });

    let redis = redis::Client::open(config.redis.endpoint.clone())?;
    logger::LogMessage::now(&instance.to_string(), logger::Data::Event {
        data: logger::Event::Startup {
            message: &format!("redis client opening @ {}", &config.redis.endpoint),
        },
    });
    if let Err(e) = redis.get_connection() {
        let e = &format!("could not connect to {}, {}", &config.redis.endpoint, e.to_string());
        logger::LogMessage::now(&instance.to_string(), logger::Data::Event {
            data: logger::Event::Error { err: e },
        });
        return Err(Box::new(error::StartupError::new(e)));
    }

    logger::LogMessage::now(&instance.to_string(), logger::Data::Event {
        data: logger::Event::Startup {
            message: &format!("nats client opening @ {}", &config.nats.endpoint),
        },
    });
    let nc = match nats::connect(&config.nats.endpoint) {
        | Ok(v) => v,
        | Err(e) => {
            logger::LogMessage::now(&instance.to_string(), logger::Data::Event {
                data: logger::Event::Error { err: &e.to_string() },
            });
            return Err(Box::new(error::StartupError::new(&e.to_string())));
        },
    };
    let nc2 = nats::jetstream::new(nc);

    let bind = config.server.address.clone();
    logger::LogMessage::now(&instance.to_string(), logger::Data::Event {
        data: logger::Event::Startup {
            message: &format!("instance will bind @ {}", &bind),
        },
    });

    let server = Server::new(config.clone(), instance.clone(), redis.clone(), nc2.clone()).start();
    HttpServer::new(move || {
        App::new()
            .service(crate::handlers::websocket::handler)
            .app_data(Data::new(server.clone()))
            .app_data(Data::new(instance.clone()))
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(redis.clone()))
            .service(crate::handlers::connection::handle_message)
            .app_data(Data::new(server.clone()))
            .app_data(Data::new(config.clone()))
            .service(crate::handlers::connection::handle_disconnect)
            .app_data(Data::new(server.clone()))
            .app_data(Data::new(config.clone()))
            .service(crate::handlers::health::handler)
    })
    .bind(&bind)?
    // .disable_signals()
    .run()
    .await?;
    Ok(())
}
