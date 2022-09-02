use actix::Addr;
use actix_web::{
    post,
    web::{
        Data,
        Payload,
    },
    Error,
    HttpRequest,
    HttpResponse,
};
use futures::StreamExt;

use crate::server::Server;

/// Endpoint for sending a message to the given connection, whether this
/// instance is the owner of it or not is irrelevant as it is an async pub/sub
/// process in the background.
#[post("/connections/{connection_id}/_send")]
pub async fn handle_message(
    _req: HttpRequest,
    mut stream: Payload,
    path: actix_web::web::Path<String>,
    srv: Data<Addr<Server>>,
    config: Data<crate::config::Config>,
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
    _req: HttpRequest,
    mut stream: Payload,
    path: actix_web::web::Path<String>,
    srv: Data<Addr<Server>>,
    config: Data<crate::config::Config>,
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
