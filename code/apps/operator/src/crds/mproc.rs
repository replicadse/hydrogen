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
            DeploymentStrategy,
        },
        autoscaling::v1::{
            CrossVersionObjectReference,
            HorizontalPodAutoscaler,
            HorizontalPodAutoscalerSpec,
        },
        core::v1::{
            Container,
            KeyToPath,
            PodSpec,
            PodTemplateSpec,
            Secret,
            SecretVolumeSource,
            Volume,
            VolumeMount,
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
use crate::{
    crds::set_annotation,
    error::WKError,
};

#[derive(Debug, PartialEq, Clone, kube::CustomResource, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[kube(
    group = "hydrogen.voidpointergroup.com",
    version = "v1",
    kind = "Mproc",
    singular = "mproc",
    plural = "mprocs",
    derive = "PartialEq",
    namespaced
)]
#[serde(rename_all = "snake_case")]
pub struct MprocSpec {
    pub hpa: MprocHpa,
    pub config: MprocSpecConfig,
}
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MprocHpa {
    pub min: i32,
    pub max: i32,
    pub cpu: i32,
}
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MprocSpecConfig {
    Inline(String),
    FromSecret { name: String },
}

#[async_trait::async_trait]
impl CRD<Mproc, MprocSpec> for Mproc {
    async fn reconcile(resource: Arc<Mproc>, context: Arc<Context>) -> Result<Action, WKError> {
        let action = resource.determine_action(resource.clone()).await?;
        println!("{:?}", action);
        match action {
            | CRDAction::NoOp => Ok(Action::requeue(Duration::from_secs(10))),
            | CRDAction::Create => {
                resource
                    .create_components(context.clone().client.clone(), &resource.spec, resource.clone())
                    .await?;

                set_annotation::<Mproc>(
                    context.clone().client.clone(),
                    resource.clone(),
                    "op_last_sync_gen",
                    "1",
                )
                .await?;
                set_finalizers::<Mproc>(context.clone().client.clone(), resource.clone(), &vec![
                    Self::finalizer_name(),
                ])
                .await?;

                Ok(Action::requeue(Duration::from_secs(10)))
            },
            | CRDAction::Delete => {
                resource
                    .delete_components(context.clone().client.clone(), resource.clone())
                    .await?;
                set_finalizers::<Mproc>(context.clone().client.clone(), resource.clone(), &vec![]).await?;
                Ok(Action::await_change())
            },
            | CRDAction::Recreate => {
                resource
                    .delete_components(context.clone().client.clone(), resource.clone())
                    .await?;
                resource
                    .create_components(context.clone().client.clone(), &resource.spec, resource.clone())
                    .await?;

                let current_gen = resource
                    .meta()
                    .generation
                    .ok_or(WKError::Generic("can not retrieve generation".to_owned()))?;
                set_annotation::<Mproc>(
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

    fn finalizer_name() -> String {
        "mprocs.hydrogen.voidpointergroup.com/finalizer".to_owned()
    }
}

impl Mproc {
    async fn determine_action(&self, resource: Arc<Mproc>) -> Result<CRDAction, WKError> {
        let was_created = resource
            .meta()
            .finalizers
            .clone()
            .map_or(vec![], |v| v)
            .contains(&Self::finalizer_name());

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

    async fn create_components(&self, client: Client, args: &MprocSpec, resource: Arc<Mproc>) -> Result<(), WKError> {
        let ns = resource
            .namespace()
            .ok_or(WKError::Generic("can not get namespace".to_owned()))?;

        match &args.config {
            | MprocSpecConfig::Inline(v) => {
                let mut sd = BTreeMap::<String, String>::new();
                sd.insert("config".to_owned(), v.to_owned());
                self.create_secret(
                    client.clone(),
                    args,
                    resource.clone(),
                    &ns,
                    &format!("{}-{}", resource.name_any(), "config"),
                    sd,
                )
                .await?;
            },
            | MprocSpecConfig::FromSecret { .. } => {},
        }
        self.create_deployment(client.clone(), args, resource.clone(), &ns)
            .await?;
        self.create_hpa(client.clone(), args, resource.clone(), &ns).await?;

        Ok(())
    }

    async fn create_deployment(
        &self,
        client: Client,
        args: &MprocSpec,
        resource: Arc<Mproc>,
        ns: &str,
    ) -> Result<(), WKError> {
        let mut labels = BTreeMap::<String, String>::new();
        labels.insert("app".to_owned(), resource.name_any().to_owned());

        let secret_name = match &args.config {
            | MprocSpecConfig::Inline(_) => format!("{}-{}", resource.name_any(), "config"),
            | MprocSpecConfig::FromSecret { name } => name.to_owned(),
        };

        let deployment = Deployment {
            metadata: ObjectMeta {
                name: Some(resource.name_any()),
                namespace: Some(ns.to_owned().clone()),
                labels: Some(labels.clone()),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                selector: LabelSelector {
                    match_labels: Some(labels.clone()),
                    match_expressions: None,
                },
                strategy: Some(DeploymentStrategy {
                    type_: Some("RollingUpdate".to_owned()),
                    ..Default::default()
                }),
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(labels),
                        ..Default::default()
                    }),
                    spec: Some(PodSpec {
                        containers: vec![Container {
                            name: resource.name_any(),
                            image: Some(
                                "harbor.chinook.k8s.voidpointergroup.com/hydrogen/hydrogen-mproc:nightly".to_owned(),
                            ),
                            ports: None,
                            volume_mounts: Some(vec![VolumeMount {
                                name: "config".to_owned(),
                                read_only: Some(true),
                                mount_path: "/app/config".to_owned(),
                                ..Default::default()
                            }]),
                            ..Default::default()
                        }],
                        volumes: Some(vec![Volume {
                            name: "config".to_owned(),
                            secret: Some(SecretVolumeSource {
                                secret_name: Some(secret_name),
                                optional: Some(false),
                                items: Some(vec![KeyToPath {
                                    key: "config".to_owned(),
                                    path: "config.yaml".to_owned(),
                                    ..Default::default()
                                }]),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        };

        let api = Api::<Deployment>::namespaced(client, &ns);
        api.create(&PostParams::default(), &deployment)
            .await
            .or_else(|e| Err(WKError::Generic(e.to_string())))?;
        Ok(())
    }

    async fn create_hpa(
        &self,
        client: Client,
        args: &MprocSpec,
        resource: Arc<Mproc>,
        ns: &str,
    ) -> Result<(), WKError> {
        let hpa = HorizontalPodAutoscaler {
            metadata: ObjectMeta {
                name: Some(resource.name_any()),
                namespace: Some(ns.to_owned()),
                ..Default::default()
            },
            spec: Some(HorizontalPodAutoscalerSpec {
                min_replicas: Some(args.hpa.min),
                max_replicas: args.hpa.max,
                scale_target_ref: CrossVersionObjectReference {
                    api_version: Some("apps/v1".to_owned()),
                    kind: "Deployment".to_owned(),
                    name: resource.name_any(),
                },
                target_cpu_utilization_percentage: Some(args.hpa.cpu),
            }),
            ..Default::default()
        };

        let api = Api::<HorizontalPodAutoscaler>::namespaced(client, &ns);
        api.create(&PostParams::default(), &hpa)
            .await
            .or_else(|e| Err(WKError::Generic(e.to_string())))?;
        Ok(())
    }

    async fn create_secret(
        &self,
        client: Client,
        _args: &MprocSpec,
        _resource: Arc<Mproc>,
        ns: &str,
        name: &str,
        data: BTreeMap<String, String>,
    ) -> Result<(), WKError> {
        let secret = Secret {
            metadata: ObjectMeta {
                name: Some(name.to_owned()),
                namespace: Some(ns.to_owned()),
                ..Default::default()
            },
            string_data: Some(data),
            immutable: Some(false),
            ..Default::default()
        };
        let api = Api::<Secret>::namespaced(client, &ns);
        api.create(&PostParams::default(), &secret)
            .await
            .or_else(|e| Err(WKError::Generic(e.to_string())))?;
        Ok(())
    }

    async fn delete_components(&self, client: Client, resource: Arc<Mproc>) -> Result<(), WKError> {
        let ns = resource
            .namespace()
            .ok_or(WKError::Generic("can not get namespace".to_owned()))?;

        let api_hpa = Api::<HorizontalPodAutoscaler>::namespaced(client.clone(), &ns);
        api_hpa
            .delete(&resource.name_any(), &DeleteParams::default())
            .await
            .or_else(|e| Err(WKError::Generic(e.to_string())))?;

        let api_deployment = Api::<Deployment>::namespaced(client.clone(), &ns);
        api_deployment
            .delete(&resource.name_any(), &DeleteParams::default())
            .await
            .or_else(|e| Err(WKError::Generic(e.to_string())))?;

        match resource.spec.config {
            | MprocSpecConfig::Inline(_) => {
                let api_secret = Api::<Secret>::namespaced(client.clone(), &ns);
                api_secret
                    .delete(
                        &format!("{}-{}", resource.name_any(), "config"),
                        &DeleteParams::default(),
                    )
                    .await
                    .or_else(|e| Err(WKError::Generic(e.to_string())))?;
            },
            | MprocSpecConfig::FromSecret { .. } => {},
        }
        Ok(())
    }
}
