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

#[post("/send/{connection_id}")]
pub async fn handler(
    _req: HttpRequest,
    mut stream: Payload,
    actix_web::web::Path(connection_id): actix_web::web::Path<uuid::Uuid>,
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
        connection: connection_id,
        msg: String::from_utf8(body.to_vec()).unwrap(),
    });
    Ok(actix_web::HttpResponse::Ok().body(""))
}
