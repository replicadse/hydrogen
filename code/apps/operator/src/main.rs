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
            check_crds(ctx.clone()).await?;

            let gateway_controller = Controller::new(
                Api::<Gateway>::all(ctx.clone().client.clone()),
                kube::api::ListParams::default(),
            );
            let mproc_controller = Controller::new(
                Api::<Mproc>::all(ctx.clone().client.clone()),
                kube::api::ListParams::default(),
            );

            let gateway_stream = gateway_controller.run(Gateway::reconcile, on_error, ctx.clone());
            let mproc_stream = mproc_controller.run(Mproc::reconcile, on_error, ctx.clone());

            let gateway_fut = gateway_stream
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
                .fuse();
            let mproc_fut = mproc_stream
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
                .fuse();

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

async fn check_crds(ctx: Arc<Context>) -> Result<(), WKError> {
    let crd_api = Api::<CustomResourceDefinition>::all(ctx.client.clone());
    let lp = ListParams::default();
    let crds = crd_api
        .list(&lp)
        .await
        .or_else(|_| Err(WKError::InvalidCRD("can not list CRDs".to_owned())))?;

    let expected = vec![
        "gateways.hydrogen.voidpointergroup.com".to_owned(),
        "mprocs.hydrogen.voidpointergroup.com".to_owned(),
    ];
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
