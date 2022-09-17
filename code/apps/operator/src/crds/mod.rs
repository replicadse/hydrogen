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
use tokio::time::Duration;

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
    async fn create_components(&self, client: Client, resource: Arc<TRes>) -> Result<(), WKError>;
    async fn delete_components(&self, client: Client, resource: Arc<TRes>) -> Result<(), WKError>;

    fn group_name() -> String;
    fn finalizer_name() -> String;
}

pub async fn reconcile<T, A>(resource: Arc<T>, context: Arc<Context>) -> Result<Action, WKError>
where
    T: CRD<T, A>+Debug+Clone+Resource+DeserializeOwned,
    T::DynamicType: Default,
{
    let action = determine_action(resource.clone(), &T::finalizer_name()).await?;
    match action {
        | CRDAction::NoOp => Ok(Action::requeue(Duration::from_secs(10))),
        | CRDAction::Create => {
            resource
                .create_components(context.clone().client.clone(), resource.clone())
                .await?;

            set_annotation::<T>(
                context.clone().client.clone(),
                resource.clone(),
                "op_last_sync_gen",
                "1",
            )
            .await?;
            set_finalizers::<T>(context.clone().client.clone(), resource.clone(), &vec![
                T::finalizer_name(),
            ])
            .await?;

            Ok(Action::requeue(Duration::from_secs(10)))
        },
        | CRDAction::Delete => {
            resource
                .delete_components(context.clone().client.clone(), resource.clone())
                .await?;
            set_finalizers::<T>(context.clone().client.clone(), resource.clone(), &vec![]).await?;
            Ok(Action::await_change())
        },
        | CRDAction::Recreate => {
            resource
                .delete_components(context.clone().client.clone(), resource.clone())
                .await?;
            resource
                .create_components(context.clone().client.clone(), resource.clone())
                .await?;

            let current_gen = resource
                .meta()
                .generation
                .ok_or(WKError::Generic("can not retrieve generation".to_owned()))?;
            set_annotation::<T>(
                context.clone().client.clone(),
                resource.clone(),
                "op_last_sync_gen",
                &current_gen.to_string(),
            )
            .await?;
            Ok(Action::requeue(Duration::from_secs(10)))
        },
    }
}

async fn determine_action<T: Resource>(resource: Arc<T>, finalizer_name: &String) -> Result<CRDAction, WKError> {
    let was_created = resource
        .meta()
        .finalizers
        .clone()
        .map_or(vec![], |v| v)
        .contains(finalizer_name);

    if was_created && resource.meta().deletion_timestamp.is_some() {
        // was created and is queued for deletion
        Ok(CRDAction::Delete)
    } else if was_created {
        // was created, is not scheduled for deletion
        let last_sync_gen = &resource
            .annotations()
            .get("op_last_sync_gen")
            .ok_or(WKError::Generic("can not retrieve generation".to_owned()))?
            .parse::<i64>()
            .or_else(|e| Err(WKError::Generic(e.to_string())))?;
        let curr_gen = resource
            .meta()
            .generation
            .ok_or(WKError::Generic("can not retrieve generation".to_owned()))?;

        if *last_sync_gen < curr_gen {
            // last sync is before the current gen
            Ok(CRDAction::Recreate)
        } else {
            // last sync is equal to the current gen
            Ok(CRDAction::NoOp)
        }
    } else if resource.meta().deletion_timestamp.is_some() {
        // was not created and is already scheduled for deletion
        Ok(CRDAction::NoOp)
    } else {
        // was not yet created
        Ok(CRDAction::Create)
    }
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
