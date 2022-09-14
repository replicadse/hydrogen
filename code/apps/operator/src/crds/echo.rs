pub async fn deploy(
    client: kube::Client,
    name: &str,
    replicas: i32,
    namespace: &str,
) -> Result<k8s_openapi::api::apps::v1::Deployment, crate::error::WKError> {
    let mut labels = std::collections::BTreeMap::<String, String>::new();
    labels.insert("app".to_owned(), name.to_owned());

    // Definition of the deployment. Alternatively, a YAML representation could be
    // used as well.
    let deployment = k8s_openapi::api::apps::v1::Deployment {
        metadata: kube::api::ObjectMeta {
            name: Some(name.to_owned()),
            namespace: Some(namespace.to_owned()),
            labels: Some(labels.clone()),
            ..kube::api::ObjectMeta::default()
        },
        spec: Some(k8s_openapi::api::apps::v1::DeploymentSpec {
            replicas: Some(replicas),
            selector: k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector {
                match_expressions: None,
                match_labels: Some(labels.clone()),
            },
            template: k8s_openapi::api::core::v1::PodTemplateSpec {
                spec: Some(k8s_openapi::api::core::v1::PodSpec {
                    containers: vec![k8s_openapi::api::core::v1::Container {
                        name: name.to_owned(),
                        image: Some("inanimate/echo-server:latest".to_owned()),
                        ports: Some(vec![k8s_openapi::api::core::v1::ContainerPort {
                            container_port: 8080,
                            ..k8s_openapi::api::core::v1::ContainerPort::default()
                        }]),
                        ..k8s_openapi::api::core::v1::Container::default()
                    }],
                    ..k8s_openapi::api::core::v1::PodSpec::default()
                }),
                metadata: Some(kube::api::ObjectMeta {
                    labels: Some(labels),
                    ..kube::api::ObjectMeta::default()
                }),
            },
            ..k8s_openapi::api::apps::v1::DeploymentSpec::default()
        }),
        ..k8s_openapi::api::apps::v1::Deployment::default()
    };

    // Create the deployment defined above
    let deployment_api = kube::Api::<k8s_openapi::api::apps::v1::Deployment>::namespaced(client, namespace);
    deployment_api
        .create(&kube::api::PostParams::default(), &deployment)
        .await
        .or_else(|_| Err(crate::error::WKError::Unknown))
}

/// Deletes an existing deployment.
///
/// # Arguments:
/// - `client` - A Kubernetes client to delete the Deployment with
/// - `name` - Name of the deployment to delete
/// - `namespace` - Namespace the existing deployment resides in
///
/// Note: It is assumed the deployment exists for simplicity. Otherwise returns
/// an Error.
pub async fn delete(client: kube::Client, name: &str, namespace: &str) -> Result<(), crate::error::WKError> {
    let api: kube::Api<k8s_openapi::api::apps::v1::Deployment> = kube::Api::namespaced(client, namespace);
    api.delete(name, &kube::api::DeleteParams::default())
        .await
        .or_else(|_| Err(crate::error::WKError::Unknown))?;
    Ok(())
}
