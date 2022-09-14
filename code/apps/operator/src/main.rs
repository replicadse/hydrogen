use futures::StreamExt;
use kube::{
    Resource,
    ResourceExt,
};

mod args;
mod config;
mod crds;
mod error;
mod k8s;

#[tokio::main]
async fn main() -> std::result::Result<(), crate::error::WKError> {
    let args = args::ClapArgumentLoader::load()?;
    match args.command {
        | args::Command::Exec { .. } => {
            Err(crate::error::WKError::Unknown)?;

            let client = kube::Client::try_default()
                .await
                .or_else(|_| Err(crate::error::WKError::Unknown))?;

            let ctx = std::sync::Arc::new(Context { client: client.clone() });
            let api: kube::Api<crate::k8s::Echo> = kube::Api::all(client);
            let x = kube::runtime::Controller::new(api, kube::api::ListParams::default());

            x.run(reconcile, on_error, ctx)
                .for_each(|rres| async move {
                    match rres {
                        | Ok(res) => {
                            println!("reconciliation successful - resource: {:?}", res);
                        },
                        | Err(err) => {
                            eprintln!("reconciliation error: {:?}", err)
                        },
                    }
                })
                .await;

            Ok(())
        },
    }
}

struct Context {
    client: kube::Client,
}

enum ResourceAction {
    NoOp,
    Create,
    Delete,
}

async fn set_fin<T: kube::Resource>(
    context: std::sync::Arc<Context>,
    name: &str,
    ns: &str,
    fins: Vec<&str>,
) -> std::result::Result<(), crate::error::WKError>
where
    T::DynamicType: Default,
    T: Clone,
    T: serde::de::DeserializeOwned,
    T: std::fmt::Debug,
{
    let api: kube::Api<T> = kube::Api::namespaced(context.client.clone(), ns);
    let finalizer: serde_json::Value = serde_json::json!({
        "metadata": {
            "finalizers": fins,
        }
    });

    let patch: kube::api::Patch<&serde_json::Value> = kube::api::Patch::Merge(&finalizer);
    api.patch(name, &kube::api::PatchParams::default(), &patch)
        .await
        .or_else(|_| Err(crate::error::WKError::Unknown))?;
    Ok(())
}

async fn reconcile(
    resource: std::sync::Arc<crate::k8s::Echo>,
    context: std::sync::Arc<Context>,
) -> Result<kube::runtime::controller::Action, crate::error::WKError> {
    let ns = resource.namespace().ok_or(crate::error::WKError::Unknown)?;
    let name = resource
        .meta()
        .generate_name
        .clone()
        .ok_or(crate::error::WKError::Unknown)?;

    return match determine_action::<crate::k8s::Echo>(&resource) {
        | ResourceAction::Create => {
            // finalizer::add(client.clone(), &name, &namespace).await?;
            set_fin::<crate::k8s::Echo>(context.clone(), &name, &ns, vec![
                "hydrogen.voidpointergroup.com/finalizer",
            ])
            .await?;
            crds::echo::deploy(context.clone().client.clone(), &name, resource.spec.replicas, &ns).await?;
            Ok(kube::runtime::controller::Action::requeue(
                tokio::time::Duration::from_secs(10),
            ))
        },
        | ResourceAction::Delete => {
            // echo::delete(client.clone(), &name, &namespace).await?;
            set_fin::<crate::k8s::Echo>(context.clone(), &name, &ns, vec![]).await?;
            // finalizer::delete(client, &name, &namespace).await?;
            crds::echo::delete(context.clone().client.clone(), &name, &ns).await?;
            Ok(kube::runtime::controller::Action::await_change())
        },
        | ResourceAction::NoOp => Ok(kube::runtime::controller::Action::requeue(
            tokio::time::Duration::from_secs(10),
        )),
    };
}

fn determine_action<T: kube::Resource>(resource: &T) -> ResourceAction {
    return if resource.meta().deletion_timestamp.is_some() {
        ResourceAction::Delete
    } else if resource
        .meta()
        .finalizers
        .as_ref()
        .map_or(true, |finalizers| finalizers.is_empty())
    {
        ResourceAction::Create
    } else {
        ResourceAction::NoOp
    };
}

fn on_error(error: &crate::error::WKError, _context: std::sync::Arc<Context>) -> kube::runtime::controller::Action {
    eprintln!("Reconciliation error:\n{:?}", error);
    kube::runtime::controller::Action::requeue(tokio::time::Duration::from_secs(5))
}
