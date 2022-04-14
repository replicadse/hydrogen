# Helm installation

This chart can be used to install `spoderman` in a Kubernetes Cluster via [Helm v3](https://helm.sh).\

## Installation

To install, follow these steps:

1) Clone the repo and `cd` into this directory
2) Run `helm dependency update`
3) Recommended but optional:\
Patch the values in `values.yaml` file (usernames & passwords, persistence, ...).
4) Run `helm install -n $NAMESPACE --create-namespace $RELEASE .`\
You need to replace these variables with your configuration values.
