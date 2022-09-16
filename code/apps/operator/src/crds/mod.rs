use std::{
    collections::BTreeMap,
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
        ListParams,
        Patch,
        PatchParams,
        PostParams,
    },
    core::ObjectMeta,
    Api,
    Client,
};
use serde_json::Value;

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
pub trait ResourceX<TArgs> {
    async fn exists(client: Client, name: &str, namespace: &str) -> Result<bool, WKError>;

    async fn create(client: Client, args: &TArgs, name: &str, namespace: &str) -> Result<(), WKError>;
    async fn delete(client: Client, name: &str, namespace: &str) -> Result<(), WKError>;

    async fn set_fin(client: Client, name: &str, namespace: &str, fins: &Vec<String>) -> Result<(), WKError>;
}

#[async_trait::async_trait]
impl ResourceX<EchoSpec> for Arc<Echo> {
    async fn exists(client: Client, name: &str, namespace: &str) -> Result<bool, WKError> {
        let api = Api::<Deployment>::namespaced(client, namespace);
        let lp = ListParams {
            field_selector: Some(format!("metadata.name={}", name)),
            ..Default::default()
        };
        let resources = api.list(&lp).await.or_else(|e| Err(WKError::Generic(e.to_string())))?;
        Ok(resources.items.len() > 0)
    }

    async fn create(client: Client, args: &EchoSpec, name: &str, namespace: &str) -> Result<(), WKError> {
        let mut labels = BTreeMap::<String, String>::new();
        labels.insert("app".to_owned(), name.to_owned());

        let deployment = Deployment {
            metadata: ObjectMeta {
                name: Some(name.to_owned()),
                namespace: Some(namespace.to_owned()),
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
                            name: name.to_owned(),
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

        // Create the deployment defined above
        let api = Api::<Deployment>::namespaced(client, namespace);
        api.create(&PostParams::default(), &deployment)
            .await
            .or_else(|e| Err(WKError::Generic(format!("can not create resource: {:?}", e))))?;
        Ok(())
    }

    async fn delete(client: Client, name: &str, namespace: &str) -> Result<(), WKError> {
        let api = Api::<Deployment>::namespaced(client, namespace);
        api.delete(name, &DeleteParams::default())
            .await
            .or_else(|e| Err(WKError::Generic(e.to_string())))?;
        Ok(())
    }

    async fn set_fin(client: Client, name: &str, namespace: &str, fins: &Vec<String>) -> Result<(), WKError> {
        let api: Api<Echo> = Api::namespaced(client, namespace);
        let finalizer: Value = serde_json::json!({
            "metadata": {
                "finalizers": fins,
            }
        });

        let patch: Patch<&Value> = Patch::Merge(&finalizer);
        api.patch(name, &PatchParams::default(), &patch)
            .await
            .or_else(|e| Err(WKError::Generic(e.to_string())))?;
        Ok(())
    }
}
