# Configuration

## Example configuration

```
version: 0.1.0

group_id: "0x0001"

server:
  address: "0.0.0.0:8080"
  heartbeat_interval_sec: 10
  connection_timeout_sec: 31
  stats_interval_sec: 10
  max_out_message_size: 262144 # 256kb
  comms:
    bidi:
      stream:
        endpoint: "nats://hydrogen-nats:4222"
        name: "hydrogen"

redis:
  endpoint: "redis://hydrogen-redis-master:6379"

routes:
  authorizer:
    endpoint: "http://hydrogen-dss-authorizer:8080"
    headers:
      Authorization: dss-authorizer-key
  connect:
    endpoint: "http://hydrogen-dss-connect:8080"
    headers:
      Authorization: dss-connect-key
  disconnect:
    endpoint: "http://hydrogen-dss-disconnect:8080"
    headers:
      Authorization: dss-disconnect-key
```

## Schema

|Key|Required|Description|Type|Example|
|-- |-- |-- |-- |-- |
|version|yes|The version of this config.|semver v2 compatible string|`1.0.0`|
|group_id|yes|An identifier for grouping multiple instances.|semver v2 compatible string|`1.0.0`|
|server|yes|The server configuration.|object||
|server.address|yes|The address to which the server binds.|$host:$port string|`0.0.0.0:8080`|
|server.heartbeat_interval_sec|yes|The duration (in seconds) between heartbeats the client has to answer. This must be less than the timeout duration `server.connection_timeout_sec`.|u16|`30`|
|server.connection_timeout_sec|yes|The duration (in seconds) when a connection times out after missing heartbeats.|u16|`60`|
|server.stats_interval_sec|no|The seconds in between stats reporting. No stats are reported if key is missing.|u16|`30`|
|server.max_out_message_size|yes|The maximum message size in bytes the server will accept from the client.|u64|`262144` (=256*1024)|
|server.comms|yes|Communication mode of the server.|object|`bidi` or `uni_server_to_client`|
|server.comms.uni_server_to_client|no|Marks server as server to client messages only.|empty object||
|server.comms.bidi|no|Makes server support bidirectional messages.|object||
|server.comms.bidi.stream|no|Information about the message stream to use (NATS/JetStream).|object||
|server.comms.bidi.stream.endpoint|yes|The endpoint on which to connect to `NATS`.|URL string|`nats://hydrogen-nats:4222`|
|server.comms.bidi.stream.name|yes|The stream name that will be used for client message brokering.|string|`hydrogen`|
|redis|yes|The `redis` configuration.|object||
|redis.endpoint|yes|The endpoint on which to connect to `redis`.|URL string|`redis://hydrogen-redis-master:6379`|
|routes|yes|The downstream service routes.|object||
|routes.authorizer|no|The authorizer downstream service.|object||
|routes.authorizer.endpoint|yes|The authorizer endpoint.|URL string|`http://hydrogen-dss-authorizer:8080`|
|routes.authorizer.headers|yes|Headers to send to the authorizer on invocation.|Map<String, String>||
|routes.connect|no|The connect downstream service.|object||
|routes.connect.endpoint|yes|The connect endpoint.|URL string|`http://hydrogen-dss-connect:8080`|
|routes.connect.headers|yes|Headers to send to the connect dss on invocation.|Map<String, String>||
|routes.disconnect|no|The disconnect downstream service.|object||
|routes.disconnect.endpoint|yes|The disconnect endpoint.|URL string|`http://hydrogen-dss-disconnect:8080`|
|routes.disconnect.headers|yes|Headers to send to the disconnect dss on invocation.|Map<String, String>||
