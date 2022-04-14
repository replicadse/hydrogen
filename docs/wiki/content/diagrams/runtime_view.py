from diagrams import Cluster, Diagram, Edge
from diagrams.k8s.network import Service
from diagrams.k8s.compute import Pod

with Diagram("runtime view"):
    with Cluster("clients"):
        clients = [
            Service("client A"),
            Service("client B")
        ]

    with Cluster("spoderman"):
        svc_spoderman = Service("spoderman")
        with Cluster("instances"):
            pods_spodermen = [
                Pod("instance A"),
                Pod("instance B")
            ]
        svc_spoderman >> pods_spodermen
        with Cluster("support"):
            svc_redis = Service("redis")
        svc_redis >> Edge(svc_redis, xlabel="sub") >> pods_spodermen
        pods_spodermen >> Edge(svc_redis, xlabel="pub") >> svc_redis

    clients >> svc_spoderman

    with Cluster("downstream dependencies"):
        svc_auth = Service("authorizer")
        svc_on_connect = Service("on connect")
        svc_rules_engine = Service("rules engine")
        svc_on_disconnect = Service("on disconnect")
    pods_spodermen >> svc_auth
    pods_spodermen >> svc_on_connect
    pods_spodermen >> svc_rules_engine
    pods_spodermen >> svc_on_disconnect

    with Cluster("business logic services"):
        services_in = [
            Service("service A"),
            Service("service B")
        ]
        services_out = [
            Service("service C"),
        ]
    
    for p in pods_spodermen:
        p >> services_in
    svc_spoderman << services_out
