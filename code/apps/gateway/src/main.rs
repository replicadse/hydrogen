mod args;
mod config;
mod error;
mod logger;
mod messages;
mod routes;
mod server;
mod types;
mod ws;
mod handlers {
    pub mod connection;
    pub mod health;
    pub mod websocket;
}

use std::error::Error;

use actix::Actor;
use actix_web::{
    http,
    web::{
        self,
        Data,
    },
    App,
    HttpServer,
};
use nats::jetstream::JetStream;
use server::Server;

use crate::types::{
    Endpoint,
    GroupID,
    InstanceID,
};

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

/// Main server function, starting an actix HTTP server with the various
/// endpoints.
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
        return Err(Box::new(crate::error::StartupError::new(e)));
    }

    let js = match config.server.comms {
        | crate::config::CommsMode::UniServerToClient => Result::<Option<JetStream>, Box<dyn Error>>::Ok(None),
        | crate::config::CommsMode::Bidi { ref stream } => {
            logger::LogMessage::now(&instance.to_string(), logger::Data::Event {
                data: logger::Event::Startup {
                    message: &format!("nats client opening @ {}", stream.endpoint),
                },
            });
            match nats::connect(&stream.endpoint) {
                | Ok(v) => Ok(Some(nats::jetstream::new(v))),
                | Err(e) => {
                    logger::LogMessage::now(&instance.to_string(), logger::Data::Event {
                        data: logger::Event::Error { err: &e.to_string() },
                    });
                    return Err(Box::new(crate::error::StartupError::new(&e.to_string())));
                },
            }
        },
    }?;

    let bind = config.server.address.clone();
    logger::LogMessage::now(&instance.to_string(), logger::Data::Event {
        data: logger::Event::Startup {
            message: &format!("instance will bind @ {}", &bind),
        },
    });

    let server = Server::new(config.clone(), instance.clone(), redis.clone(), js.clone()).start();
    HttpServer::new(move || {
        let mut app = App::new()
            .app_data(Data::new(server.clone()))
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(InstanceID::from(instance.clone())))
            .app_data(Data::new(GroupID::from(config.group_id.clone())))
            .service(crate::handlers::connection::handle_broadcast_message)
            .service(crate::handlers::connection::handle_server_message)
            .service(crate::handlers::connection::handle_disconnect)
            .service(crate::handlers::health::handler);

        for ep in config.clone().routes.endpoints {
            assert!(ep.starts_with("/"), "routes need to start with a forward slash");

            app = app.service(
                web::resource(format!("/ws{}", ep))
                    .route(
                        web::get()
                            .method(http::Method::GET)
                            .to(crate::handlers::websocket::handler),
                    )
                    .app_data(Data::new(Endpoint::from(ep.clone()))),
            )
        }
        app
    })
    .bind(&bind)?
    .run()
    .await?;
    Ok(())
}
