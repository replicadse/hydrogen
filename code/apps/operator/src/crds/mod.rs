#[derive(kube::CustomResource, serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone, schemars::JsonSchema)]
#[kube(
    group = "voidpointergroup.com",
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
