use std::{
    collections::HashSet,
    sync::Arc,
};

use crds::{
    gateway::Gateway,
    mproc::Mproc,
    Context,
    CRD,
};
use error::WKError;
use futures::{
    select,
    FutureExt,
    StreamExt,
};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::ListParams,
    runtime::{
        controller::Action,
        Controller,
    },
    Api,
    Client,
    Resource,
};

use crate::crds::{
    gateway::GatewaySpec,
    mproc::MprocSpec,
    reconcile,
};

mod args;
mod config;
mod crds;
mod error;

#[tokio::main]
async fn main() -> std::result::Result<(), WKError> {
    let args = args::ClapArgumentLoader::load()?;

    match args.command {
        | args::Command::Exec { .. } => {
            let client = Client::try_default()
                .await
                .or_else(|_| Err(WKError::Generic("kube client".to_owned())))?;

            let ctx = Arc::new(Context { client: client.clone() });
            check_crds(ctx.clone(), vec![Gateway::group_name(), Mproc::group_name()]).await?;

            macro_rules! start_controller {
                ($type:ident, $typeArgs:ident) => {
                    Controller::new(
                        Api::<$type>::all(ctx.clone().client.clone()),
                        ListParams::default(),
                    )
                    .run(reconcile::<$type, $typeArgs>, on_error, ctx.clone())
                    .for_each(|rres| async move {
                        match rres {
                            | Ok(res) => {
                                println!("run -> reconciliation successful - resource: {:?}", res);
                            },
                            | Err(err) => {
                                eprintln!("run -> reconciliation error: {:?}", err)
                            },
                        }
                    })
                    .fuse()
                };
            }

            let gateway_fut = start_controller!(Gateway, GatewaySpec);
            let mproc_fut = start_controller!(Mproc, MprocSpec);

            futures::pin_mut!(gateway_fut, mproc_fut);
            select! {
                () = gateway_fut => {},
                () = mproc_fut => {},
            }

            Err(WKError::Generic(
                "operator terminated - controller task error".to_owned(),
            ))
        },
    }
}

async fn check_crds(ctx: Arc<Context>, expected: Vec<String>) -> Result<(), WKError> {
    let crd_api = Api::<CustomResourceDefinition>::all(ctx.client.clone());
    let lp = ListParams::default();
    let crds = crd_api
        .list(&lp)
        .await
        .or_else(|_| Err(WKError::InvalidCRD("can not list CRDs".to_owned())))?;

    let mut matches = HashSet::<String>::new();
    for crd in crds {
        match crd.meta().name.clone() {
            | Some(v) => {
                if expected.contains(&v) {
                    matches.insert(v);
                }
            },
            | None => {},
        }
    }

    if expected.len() == matches.len() {
        Ok(())
    } else {
        Err(WKError::InvalidCRD("missing CRDs".to_owned()))
    }
}

fn on_error(error: &WKError, _context: Arc<Context>) -> Action {
    eprintln!("on_error -> reconciliation error: {:?}", error);
    Action::requeue(tokio::time::Duration::from_secs(5))
}
