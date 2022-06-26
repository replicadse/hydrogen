use actix::Addr;
use actix_web::{
    get,
    web::{
        Data,
        Payload,
    },
    Error,
    HttpRequest,
    HttpResponse,
};
use actix_web_actors::ws;

use crate::{
    server::Server,
    ws::WsConn,
};

#[get("/ws")]
pub async fn handler(
    req: HttpRequest,
    stream: Payload,
    srv: Data<Addr<Server>>,
    instance: Data<uuid::Uuid>,
    config: Data<crate::config::Config>,
) -> Result<HttpResponse, Error> {
    let safecall_auth = |conn_id: &uuid::Uuid,
                         auth_route: &std::option::Option<crate::config::Authorizer>|
     -> Result<(), Box<dyn std::error::Error>> {
        match auth_route {
            | Some(c) => {
                let mut auth_req = ureq::post(&c.endpoint);
                for (k, v) in c.headers.iter() {
                    auth_req = auth_req.set(k, v);
                }
                let resp = auth_req.send_string(&serde_json::to_string(&crate::routes::AuthorizerRequest {
                    instance_id: &instance.to_string(),
                    connection_id: &conn_id.to_string(),
                    time: &chrono::Utc::now().to_rfc3339(),
                    headers: c.headers.iter().map(|v| (v.0.to_owned(), v.1.to_owned())).collect(),
                })?)?;

                crate::logger::LogMessage::now(&instance.to_string(), crate::logger::Data::Event {
                    data: crate::logger::Event::AuthRouteResponse {
                        connection: &conn_id.to_string(),
                        response: resp.status(),
                    },
                });

                match resp.status() {
                    | 200 => Ok(()),
                    | _ => Err(Box::new(crate::error::ConnectRouteError::new(&format!(
                        "authorizer route error code {}",
                        resp.status()
                    )))),
                }
            },
            | None => Ok(()),
        }
    };

    let ws = WsConn::new(
        srv.get_ref().clone(),
        std::time::Duration::from_secs(config.server.heartbeat_interval_sec.into()),
        std::time::Duration::from_secs(config.server.connection_timeout_sec.into()),
    );
    match safecall_auth(&ws.connection, &config.routes.authorizer) {
        | Ok(_) => {
            let resp = ws::start(ws, &req, stream)?;
            Ok(resp)
        },
        | Err(e) => {
            crate::logger::LogMessage::now(&instance.to_string(), crate::logger::Data::Event {
                data: crate::logger::Event::Error { err: &e.to_string() },
            });
            Err(actix_web::error::ErrorUnauthorized(e))
        },
    }
}
