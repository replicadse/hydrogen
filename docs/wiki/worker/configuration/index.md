# Configuration

## Example configuration

```
version: 0.1.0

queue:
  nats:
    endpoint: "nats://spoderman-nats:4222"
    stream: client

routes:
  rules_engine:
    endpoint: "http://spoderman-dss-rules-engine:8080"
    headers:
      Authorization: dss-rules-engine-key
```

## Schema

|Key|Required|Description|Type|Example|
|-- |-- |-- |-- |-- |
|version|yes|The version of this config.|semver v3 compatible string|`1.0.0`|
|queue|yes|The config for consuming messages.|object||
|queue.nats|yes|The `NATS` configuration.|object||
|queue.nats.endpoint|yes|The endpoint on which to connect to `NATS`.|URL string|`nats://spoderman-nats:4222`|
|queue.nats.stream|yes|The stream name that will be used for client message brokering.|string|`client`|
|routes|yes|The downstream service routes.|object||
|routes.rules_engine|no|The rules engine downstream service.|object||
|routes.rules_engine.endpoint|yes|The rules engine endpoint.|URL string|`http://spoderman-dss-disconnect:8080`|
|routes.rules_engine.headers|yes|Headers to send to the rules engine dss on invocation.|Map<String, String>||
