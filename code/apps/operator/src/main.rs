use std::{
    collections::HashSet,
    sync::Arc,
};

use crds::{
    echo::Echo,
    Context,
    CRD,
};
use error::WKError;
use futures::stream::StreamExt;
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

            let x = Controller::new(
                Api::<Echo>::all(ctx.clone().client.clone()),
                kube::api::ListParams::default(),
            );

            x.run(Echo::reconcile, on_error, ctx)
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
                .await;

            Ok(())
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

    let expected = vec!["echoes.hydrogen.voidpointergroup.com".to_owned()];
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
