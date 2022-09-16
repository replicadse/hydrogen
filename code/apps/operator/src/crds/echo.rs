use std::{
    collections::BTreeMap,
    fmt::Debug,
    sync::Arc,
};

use k8s_openapi::{
    api::{
        apps::v1::{
            Deployment,
            DeploymentSpec,
        },
        core::v1::{
            Container,
            ContainerPort,
            PodSpec,
            PodTemplateSpec,
        },
    },
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::{
    api::{
        DeleteParams,
        PostParams,
    },
    core::ObjectMeta,
    runtime::controller::Action,
    Api,
    Client,
    Resource,
    ResourceExt,
};
use tokio::time::Duration;

use super::{
    set_finalizers,
    CRDAction,
    Context,
    CRD,
};
use crate::error::WKError;

#[derive(kube::CustomResource, serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone, schemars::JsonSchema)]
#[kube(
    group = "hydrogen.voidpointergroup.com",
    version = "v1",
    kind = "Echo",
    singular = "echo",
    plural = "echoes",
    derive = "PartialEq",
    namespaced
)]
pub struct EchoSpec {
    pub replicas: i32,
}

#[async_trait::async_trait]
impl CRD<Echo, EchoSpec> for Echo {
    async fn reconcile(resource: Arc<Echo>, context: Arc<Context>) -> Result<Action, WKError> {
        let action = resource.determine_action(resource.clone()).await?;
        println!("{:?}", action);
        match action {
            | CRDAction::NoOp => Ok(Action::requeue(Duration::from_secs(10))),
            | CRDAction::Create => {
                resource
                    .create(context.clone().client.clone(), &resource.spec, resource.clone())
                    .await?;

                set_finalizers::<Echo>(context.clone().client.clone(), resource.clone(), &vec![
                    Self::finalizer_name(),
                ])
                .await?;

                Ok(Action::requeue(Duration::from_secs(10)))
            },
            | CRDAction::Delete => {
                resource
                    .delete(context.clone().client.clone(), resource.clone())
                    .await?;
                set_finalizers::<Echo>(context.clone().client.clone(), resource.clone(), &vec![]).await?;
                Ok(Action::await_change())
            },
        }
    }

    fn finalizer_name() -> String {
        "echoes.hydrogen.voidpointergroup.com/finalizer".to_owned()
    }
}

impl Echo {
    async fn determine_action(&self, resource: Arc<Echo>) -> Result<CRDAction, WKError> {
        let was_created = resource
            .meta()
            .finalizers
            .clone()
            .map_or(vec![], |v| v)
            .contains(&Self::finalizer_name());

        if was_created {
            if resource.meta().deletion_timestamp.is_some() {
                Ok(CRDAction::Delete)
            } else {
                Ok(CRDAction::NoOp)
            }
        } else {
            if resource.meta().deletion_timestamp.is_some() {
                Ok(CRDAction::NoOp)
            } else {
                Ok(CRDAction::Create)
            }
        }
    }

    async fn create(&self, client: Client, args: &EchoSpec, resource: Arc<Echo>) -> Result<(), WKError> {
        let ns = resource
            .namespace()
            .ok_or(WKError::Generic("can not get namespace".to_owned()))?;
        let api = Api::<Deployment>::namespaced(client, &ns);

        let mut labels = BTreeMap::<String, String>::new();
        labels.insert("app".to_owned(), resource.name_any().to_owned());

        let deployment = Deployment {
            metadata: ObjectMeta {
                name: Some(resource.name_any()),
                namespace: Some(ns.clone()),
                labels: Some(labels.clone()),
                ..ObjectMeta::default()
            },
            spec: Some(DeploymentSpec {
                replicas: Some(args.replicas),
                selector: LabelSelector {
                    match_expressions: None,
                    match_labels: Some(labels.clone()),
                },
                template: PodTemplateSpec {
                    spec: Some(PodSpec {
                        containers: vec![Container {
                            name: resource.name_any(),
                            image: Some("nginx:latest".to_owned()),
                            ports: Some(vec![ContainerPort {
                                container_port: 8080,
                                ..ContainerPort::default()
                            }]),
                            ..Container::default()
                        }],
                        ..PodSpec::default()
                    }),
                    metadata: Some(ObjectMeta {
                        labels: Some(labels),
                        ..ObjectMeta::default()
                    }),
                },
                ..DeploymentSpec::default()
            }),
            ..Deployment::default()
        };

        api.create(&PostParams::default(), &deployment)
            .await
            .or_else(|e| Err(WKError::Generic(e.to_string())))?;
        Ok(())
    }

    async fn delete(&self, client: Client, resource: Arc<Echo>) -> Result<(), WKError> {
        let ns = resource
            .namespace()
            .ok_or(WKError::Generic("can not get namespace".to_owned()))?;

        let api = Api::<Deployment>::namespaced(client, &ns);
        api.delete(&resource.name_any(), &DeleteParams::default())
            .await
            .or_else(|e| Err(WKError::Generic(e.to_string())))?;
        Ok(())
    }
}
