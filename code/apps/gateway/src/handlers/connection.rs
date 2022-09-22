use actix::Addr;
use actix_web::{
    self,
    post,
    web::{
        Data,
        Path,
        Payload,
    },
    Error,
    HttpRequest,
    HttpResponse,
};
use futures::StreamExt;

use crate::{
    config::Config,
    server::Server,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BroadcastQueryParams {
    pub endpoints: Option<Vec<String>>,
}

/// Endpoint for broadcasting a message to all connections.
#[post("/connections/_broadcast")]
pub async fn handle_broadcast_message(
    req: HttpRequest,
    mut stream: Payload,
    srv: Data<Addr<Server>>,
    config: Data<Config>,
) -> Result<HttpResponse, Error> {
    let q_params = serde_qs::Config::new(4, false).deserialize_str::<BroadcastQueryParams>(&req.query_string())?;
    let mut body = actix_web::web::BytesMut::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > config.server.max_out_message_size {
            return Err(actix_web::error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    match &q_params.endpoints {
        | Some(endpoints) => {
            for ep in endpoints.iter() {
                srv.do_send(crate::messages::BroadcastServerMessage::Endpoint {
                    endpoint: ep.to_owned(),
                    time: chrono::Utc::now().to_rfc3339(),
                    message: String::from_utf8(body.to_vec()).unwrap(),
                });
            }
        },
        | None => {
            srv.do_send(crate::messages::BroadcastServerMessage::All {
                time: chrono::Utc::now().to_rfc3339(),
                message: String::from_utf8(body.to_vec()).unwrap(),
            });
        },
    }
    Ok(actix_web::HttpResponse::Ok().body(""))
}

/// Endpoint for sending a message to the given connection, whether this
/// instance is the owner of it or not is irrelevant as it is an async pub/sub
/// process in the background.
#[post("/connections/{connection_id}/_send")]
pub async fn handle_server_message(
    mut stream: Payload,
    path: Path<String>,
    srv: Data<Addr<Server>>,
    config: Data<Config>,
) -> Result<HttpResponse, Error> {
    let mut body = actix_web::web::BytesMut::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > config.server.max_out_message_size {
            return Err(actix_web::error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    srv.do_send(crate::messages::ServerMessage {
        connection: path.into_inner(),
        time: chrono::Utc::now().to_rfc3339(),
        message: String::from_utf8(body.to_vec()).unwrap(),
    });
    Ok(actix_web::HttpResponse::Ok().body(""))
}

/// Endpoint for forcing the disconnect of a given connection, whether this
/// instance is the owner of it or not is irrelevant as it is an async pub/sub
/// process in the background.
#[post("/connections/{connection_id}/_disconnect")]
pub async fn handle_disconnect(
    mut stream: Payload,
    path: Path<String>,
    srv: Data<Addr<Server>>,
    config: Data<Config>,
) -> Result<HttpResponse, Error> {
    let mut body = actix_web::web::BytesMut::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > config.server.max_out_message_size {
            return Err(actix_web::error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    srv.do_send(crate::messages::ServerDisconnect {
        connection: path.into_inner(),
        time: chrono::Utc::now().to_rfc3339(),
        reason: String::from_utf8(body.to_vec()).unwrap(),
    });
    Ok(actix_web::HttpResponse::Ok().body(""))
}
