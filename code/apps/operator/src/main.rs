use std::{
    collections::HashSet,
    sync::{
        self,
        Arc,
    },
};

use crds::Echo;
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
use tokio::time::Duration;

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

            let ctx = sync::Arc::new(Context { client: client.clone() });
            check_crds(ctx.clone()).await?;

            let x = Controller::new(
                Api::<Echo>::namespaced(ctx.clone().client.clone(), "hydrogen"),
                kube::api::ListParams::default(),
            );

            x.run(reconcile, on_error, ctx)
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

    let expected = vec!["echoes.voidpointergroup.com".to_owned()];
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

struct Context {
    client: Client,
}

// async fn set_fin<T: kube::Resource>(
//     context: std::sync::Arc<Context>,
//     name: &str,
//     ns: &str,
//     fins: Vec<&str>,
// ) -> std::result::Result<(), crate::error::WKError>
// where
//     T::DynamicType: Default,
//     T: Clone,
//     T: serde::de::DeserializeOwned,
//     T: std::fmt::Debug,
// {
//     let finalizer: serde_json::Value = serde_json::json!({
//         "metadata": {
//             "finalizers": fins,
//         }
//     });

//     let patch: kube::api::Patch<&serde_json::Value> =
// kube::api::Patch::Merge(&finalizer);     context
//         .api_ns::<T>(ns)
//         .patch(name, &kube::api::PatchParams::default(), &patch)
//         .await
//         .or_else(|_| Err(crate::error::WKError::Generic("can not patch
// resource".to_owned())))?;     Ok(())
// }

async fn reconcile(_resource: sync::Arc<Echo>, _context: sync::Arc<Context>) -> Result<Action, WKError> {
    Ok(Action::requeue(Duration::from_secs(10)))
}

fn on_error(error: &WKError, _context: sync::Arc<Context>) -> Action {
    eprintln!("on_error -> reconciliation error:\n{:?}", error);
    Action::requeue(tokio::time::Duration::from_secs(5))
}
