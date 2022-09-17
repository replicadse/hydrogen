use std::{
    fmt::Debug,
    sync::Arc,
};

use kube::{
    api::{
        Patch,
        PatchParams,
    },
    runtime::controller::Action,
    Api,
    Client,
    Resource,
    ResourceExt,
};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::WKError;

pub(crate) mod gateway;
pub(crate) mod mproc;

pub struct Context {
    pub client: Client,
}

#[derive(Debug)]
pub enum CRDAction {
    NoOp,
    Create,
    Delete,
    Recreate,
}

#[async_trait::async_trait]
pub trait CRD<TRes, TArgs>
where
    TRes: Debug,
    TRes: Clone,
    TRes: Resource,
    TRes: DeserializeOwned,
{
    async fn reconcile(res: Arc<TRes>, context: Arc<Context>) -> Result<Action, WKError>;

    fn finalizer_name() -> String;
}

async fn set_finalizers<T>(client: Client, resource: Arc<T>, fins: &Vec<String>) -> Result<(), WKError>
where
    T: Debug,
    T: Clone,
    T: Resource,
    T: DeserializeOwned,
    <T as Resource>::DynamicType: Default,
{
    let ns = resource
        .namespace()
        .ok_or(WKError::Generic("can not get namespace".to_owned()))?;

    let api: Api<T> = Api::namespaced(client, &ns);
    let finalizer: Value = serde_json::json!({
        "metadata": {
            "finalizers": fins,
        }
    });

    let patch: Patch<&Value> = Patch::Merge(&finalizer);
    api.patch(&resource.name_any(), &PatchParams::default(), &patch)
        .await
        .or_else(|e| Err(WKError::Generic(e.to_string())))?;
    Ok(())
}

async fn set_annotation<T>(client: Client, resource: Arc<T>, k: &str, v: &str) -> Result<(), WKError>
where
    T: Debug,
    T: Clone,
    T: Resource,
    T: DeserializeOwned,
    <T as Resource>::DynamicType: Default,
{
    let ns = resource
        .namespace()
        .ok_or(WKError::Generic("can not get namespace".to_owned()))?;

    let api: Api<T> = Api::namespaced(client, &ns);
    let annotations: Value = serde_json::json!({
        "metadata": {
            "annotations": {
                k: v,
            },
        }
    });

    let patch: Patch<&Value> = Patch::Merge(&annotations);
    api.patch(&resource.name_any(), &PatchParams::default(), &patch)
        .await
        .or_else(|e| Err(WKError::Generic(e.to_string())))?;
    Ok(())
}
