from cProfile import label
from diagrams import Cluster, Diagram, Edge
from diagrams.k8s.network import Service
from diagrams.k8s.compute import Deployment, Pod
from diagrams.k8s.clusterconfig import HorizontalPodAutoscaler
from diagrams.k8s.ecosystem import Helm

with Diagram("deployment view"):
    with Cluster("spoderman"):
        svc_spoderman = Service("spoderman")
        dpl_spoderman = Deployment("spoderman")
        hpa_spoderman = HorizontalPodAutoscaler("spoderman")
        pods_spoderman = [
            Pod("spoderman A"),
            Pod("spoderman B"),
        ]

        svc_spoderman >> Edge(label="use") >> dpl_spoderman
        hpa_spoderman >> Edge(label="configure") >> dpl_spoderman
        dpl_spoderman >> Edge(label="spawn & kill") >> pods_spoderman

    with Cluster("redis"):
        svc_redis = Service("redis")
        for p in pods_spoderman:
            p >> Edge(label="use") >> svc_redis
