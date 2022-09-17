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
            Service,
            ServicePort,
            ServiceSpec,
            Volume,
            VolumeMount,
        },
    },
    apimachinery::pkg::{
        apis::meta::v1::LabelSelector,
        util::intstr::IntOrString,
    },
};
use kube::{
    api::{
        DeleteParams,
        PostParams,
    },
    core::ObjectMeta,
    Api,
    Client,
    ResourceExt,
};

use super::CRD;
use crate::error::WKError;

#[derive(Debug, PartialEq, Clone, kube::CustomResource, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
#[kube(
    group = "hydrogen.voidpointergroup.com",
    version = "v1",
    kind = "Gateway",
    singular = "gateway",
    plural = "gateways",
    derive = "PartialEq",
    namespaced
)]
pub struct GatewaySpec {
    pub hpa: GatewayHpa,
    pub config: GatewaySpecConfig,
}
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GatewayHpa {
    pub min: i32,
    pub max: i32,
    pub cpu: i32,
}
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GatewaySpecConfig {
    Inline(String),
    FromSecret { name: String, field: String },
}

#[async_trait::async_trait]
impl CRD<Gateway, GatewaySpec> for Gateway {
    async fn create_components(&self, client: Client, resource: Arc<Gateway>) -> Result<(), WKError> {
        let ns = resource
            .namespace()
            .ok_or(WKError::Generic("can not get namespace".to_owned()))?;

        match &resource.spec.config {
            | GatewaySpecConfig::Inline(v) => {
                let mut sd = BTreeMap::<String, String>::new();
                sd.insert("config".to_owned(), v.to_owned());
                self.create_secret(
                    client.clone(),
                    resource.clone(),
                    &ns,
                    &format!("{}-{}", resource.name_any(), "config"),
                    sd,
                )
                .await?;
            },
            | GatewaySpecConfig::FromSecret { .. } => {},
        }
        self.create_deployment(client.clone(), resource.clone(), &ns).await?;
        self.create_hpa(client.clone(), resource.clone(), &ns).await?;
        self.create_service(client.clone(), resource.clone(), &ns).await?;

        Ok(())
    }

    async fn delete_components(&self, client: Client, resource: Arc<Gateway>) -> Result<(), WKError> {
        let ns = resource
            .namespace()
            .ok_or(WKError::Generic("can not get namespace".to_owned()))?;

        let api_service = Api::<Service>::namespaced(client.clone(), &ns);
        api_service
            .delete(&resource.name_any(), &DeleteParams::default())
            .await
            .or_else(|e| Err(WKError::Generic(e.to_string())))?;

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
            | GatewaySpecConfig::Inline(_) => {
                let api_secret = Api::<Secret>::namespaced(client.clone(), &ns);
                api_secret
                    .delete(
                        &format!("{}-{}", resource.name_any(), "config"),
                        &DeleteParams::default(),
                    )
                    .await
                    .or_else(|e| Err(WKError::Generic(e.to_string())))?;
            },
            | GatewaySpecConfig::FromSecret { .. } => {},
        }
        Ok(())
    }

    fn group_name() -> String {
        "gateways.hydrogen.voidpointergroup.com".to_owned()
    }

    fn finalizer_name() -> String {
        "gateways.hydrogen.voidpointergroup.com/finalizer".to_owned()
    }
}

impl Gateway {
    async fn create_deployment(&self, client: Client, resource: Arc<Gateway>, ns: &str) -> Result<(), WKError> {
        let mut labels = BTreeMap::<String, String>::new();
        labels.insert("app".to_owned(), resource.name_any().to_owned());

        let secret = match &resource.spec.config {
            | GatewaySpecConfig::Inline(_) => (format!("{}-{}", resource.name_any(), "config"), "config".to_owned()),
            | GatewaySpecConfig::FromSecret { name, field } => (name.to_owned(), field.to_owned()),
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
                                "harbor.chinook.k8s.voidpointergroup.com/hydrogen/hydrogen-gateway:nightly".to_owned(),
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
                                secret_name: Some(secret.0),
                                optional: Some(false),
                                items: Some(vec![KeyToPath {
                                    key: secret.1,
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

    async fn create_hpa(&self, client: Client, resource: Arc<Gateway>, ns: &str) -> Result<(), WKError> {
        let hpa = HorizontalPodAutoscaler {
            metadata: ObjectMeta {
                name: Some(resource.name_any()),
                namespace: Some(ns.to_owned()),
                ..Default::default()
            },
            spec: Some(HorizontalPodAutoscalerSpec {
                min_replicas: Some(resource.spec.hpa.min),
                max_replicas: resource.spec.hpa.max,
                scale_target_ref: CrossVersionObjectReference {
                    api_version: Some("apps/v1".to_owned()),
                    kind: "Deployment".to_owned(),
                    name: resource.name_any(),
                },
                target_cpu_utilization_percentage: Some(resource.spec.hpa.cpu),
            }),
            ..Default::default()
        };

        let api = Api::<HorizontalPodAutoscaler>::namespaced(client, &ns);
        api.create(&PostParams::default(), &hpa)
            .await
            .or_else(|e| Err(WKError::Generic(e.to_string())))?;
        Ok(())
    }

    async fn create_service(&self, client: Client, resource: Arc<Gateway>, ns: &str) -> Result<(), WKError> {
        let mut selector = BTreeMap::<String, String>::new();
        selector.insert("app".to_owned(), resource.name_any());

        let service = Service {
            metadata: ObjectMeta {
                name: Some(resource.name_any()),
                namespace: Some(ns.to_owned()),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                type_: Some("ClusterIP".to_owned()),
                ports: Some(vec![ServicePort {
                    port: 8080,
                    target_port: Some(IntOrString::Int(8080)),
                    protocol: Some("TCP".to_owned()),
                    ..Default::default()
                }]),
                selector: Some(selector),
                ..Default::default()
            }),
            ..Default::default()
        };

        let api = Api::<Service>::namespaced(client, &ns);
        api.create(&PostParams::default(), &service)
            .await
            .or_else(|e| Err(WKError::Generic(e.to_string())))?;
        Ok(())
    }

    async fn create_secret(
        &self,
        client: Client,
        _resource: Arc<Gateway>,
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
}
