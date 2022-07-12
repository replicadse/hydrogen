# Configuration

## Example configuration

```
version: 0.1.0

queue:
  nats:
    endpoint: "nats://hydrogen-nats:4222"
    stream: client

engine_mode:
  regex:
    rules:
      - regex: "^!"
        route:
          endpoint: "http://hydrogen-dss-sink-b:8080"
          headers:
            Authorization: dss-sink-b-key
      - regex: ".*"
        route:
          endpoint: "http://hydrogen-dss-sink-a:8080"
          headers:
            Authorization: dss-sink-a-key
  # dss:
  #   rules_engine:
  #     endpoint: "http://hydrogen-dss-rules-engine:8080"
  #     headers:
  #       Authorization: dss-rules-engine-key

```

## Schema

|Key|Required|Description|Type|Example|
|-- |-- |-- |-- |-- |
|version|yes|The version of this config.|semver v2 compatible string|`1.0.0`|
|queue|yes|The config for consuming messages.|object||
|queue.nats|yes|The `NATS` configuration.|object||
|queue.nats.endpoint|yes|The endpoint on which to connect to `NATS`.|URL string|`nats://hydrogen-nats:4222`|
|queue.nats.stream|yes|The stream name that will be used for client message brokering.|string|`client`|
|engine_mode|yes|The engine mode details which are used to process messages.|object (enum) - needs one mode active||
|engine_mode.regex|no|Regex mode - forwarding messages by evaluating them over regular expressions.|object||
|engine_mode.regex.rules|yes|Contains the regular expressions and the routes to which they lead if they match. The expressions will be checked sequentially. If none match, the message is logged and dropped. A catch-all rule at the end is usually a good idea.|array||
|engine_mode.regex.rules.$.regex|yes|The regular that has to match expression for this destination.|regex string|"^!" for every message starting with "!" or ".*" for catching all|
|engine_mode.regex.rules.$.route|yes|The route to the message destination.|object||
|engine_mode.regex.rules.$.route.endpoint|yes|The HTTP endpoint to the message destination.|URL string|`http://hydrogen-dss-sink-a:8080`|
|engine_mode.regex.rules.$.route.headers|yes|Headers to send to the message destination on invocation.|Map<String, String>||
|engine_mode.dss|no|Downstream service mode - invokes a remote rules engine which returns the message destination (for more complex use-cases).|object||
|engine_mode.dss.rules_engine|yes|The rules engine downstream service.|object||
|engine_mode.dss.rules_engine.endpoint|yes|The rules engine endpoint.|URL string|`http://hydrogen-sink-a:8080`|
|engine_mode.dss.rules_engine.headers|yes|Headers to send to the rules engine dss on invocation.|Map<String, String>||
