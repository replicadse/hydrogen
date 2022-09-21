use actix::Addr;
use actix_web::{
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

/// Handler for initiating connection between client and server. This function
/// is only invoked at the first stage of initiation and a connection is
/// established from there. This function will also invoke the authorizer route
/// if present to determine whether the connection may or may not be
/// established. It will also enrich the context of the connection with
/// the context that is returned by the authorizer in it's response.
pub async fn handler(
    req: HttpRequest,
    stream: Payload,
    srv: Data<Addr<Server>>,
    instance: Data<String>,
    endpoint: Data<String>,
    config: Data<crate::config::Config>,
) -> Result<HttpResponse, Error> {
    let safecall_auth =
        |conn_id: &str,
         auth_route: &std::option::Option<crate::config::Authorizer>|
         -> Result<std::option::Option<crate::routes::AuthorizerResponse>, Box<dyn std::error::Error>> {
            match auth_route {
                | Some(c) => Ok(Some(invoke_authorizer_route(
                    &instance,
                    &c.endpoint,
                    &c.headers,
                    conn_id,
                )?)),
                | None => Ok(None),
            }
        };

    let ws_id = WsConn::claim_id();
    match safecall_auth(&ws_id, &config.routes.authorizer) {
        | Ok(ar) => {
            let ws = WsConn::new(
                ws_id,
                endpoint.get_ref().to_owned(),
                srv.get_ref().clone(),
                crate::ws::WsConnContext {
                    authorizer: match ar {
                        | Some(ar) => ar.context,
                        | None => None,
                    },
                },
                std::time::Duration::from_secs(config.server.heartbeat_interval_sec.into()),
                std::time::Duration::from_secs(config.server.connection_timeout_sec.into()),
            );
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

fn invoke_authorizer_route(
    instance: &str,
    endpoint: &str,
    headers: &std::collections::HashMap<String, String>,
    conn_id: &str,
) -> std::result::Result<crate::routes::AuthorizerResponse, Box<dyn std::error::Error>> {
    let mut auth_req = ureq::post(endpoint);
    for (k, v) in headers.iter() {
        auth_req = auth_req.set(k, v);
    }
    let resp = auth_req.send_string(&serde_json::to_string(&crate::routes::AuthorizerRequest {
        instance_id: instance.to_owned(),
        connection_id: conn_id.to_owned(),
        endpoint: endpoint.to_owned(),
        time: chrono::Utc::now().to_rfc3339(),
        headers: headers.iter().map(|v| (v.0.to_owned(), v.1.to_owned())).collect(),
    })?)?;
    let resp_status = resp.status();

    // TODO(AWE): instrumentation has shown that the following statement takes
    // ~1sec. This should not be the case and needs to be
    // resolved. Maybe it makes sense to go away from ureq at this point?
    let respstr = resp.into_string()?;

    let resp_parsed = serde_json::from_str::<crate::routes::AuthorizerResponse>(&respstr)?;
    crate::logger::LogMessage::now(&instance.to_string(), crate::logger::Data::Event {
        data: crate::logger::Event::AuthRouteResponse {
            connection: &conn_id.to_string(),
            response: resp_status,
        },
    });

    match resp_status {
        | 200 => Ok(resp_parsed),
        | _ => Err(Box::new(crate::error::AuthorizerRouteError::new(&format!(
            "authorizer route error code {}",
            resp_status
        )))),
    }
}
