# spoderman

`spoderman` is a kubernetes native API gateway for websocket connections that indirects messages into HTTP requests.

## Release schemes & cycles

As `spoderman` now gets more complete in it's feature set, there are final preparations to be made before releasing a v1.

There will be `1.0.0-alpha.x` releases first with the first coming up end of June 2022. These are meant to be for testing purposes only and will primarily serve for identifying which things are still missing that have not come up yet. \
Following the alpha releases, there will be a beta phase. The feature set for v1.0 will be complete upon entering the beta phase and there will only be fixes made to the existing features. At this point in time, the documentation about the software will also be much more in depth and detailed although it might not be 100% complete. \
Following the beta phase with release candidates, v1.0 will be published. Following the v1.0 release, `spoderman` will strictly adhere to the `semver v3` versioning scheme.

## Roadmap to v1

All work, features and generally everything that will be included in / done before the v1.0 release will be bundled in [this milestone](https://github.com/voidpointergroup/spoderman/milestone/1).

## v1.0 features

* Bidirectional communication (client <--> server) \
  Client messages are sent from the client, server messages are sent towards any running instance of spoderman and forwarded towards the client from the instance that holds it's connection. \
  Avoiding persistent connections towards your service is crucial to reach a near zero downtime when deploying and an increased consumer satisfaction.
* Routing service \
  No fancy DSL or other technique required. For every request, a routing service is invoked which returns the destination for that message including other metadata (like headers).
* Authorization service \
  It brings built in authorization that is performed before a persistent connection is established.
* OnConnect/OnDisconnect services \
  These are invoked when a client has connected / disconnected respectively.
* Monitoring \
  Structured log messages for events and, if configured, interval reporting about the application's state are available.
* Easy installs via HELM charts
* Multi language / framework support \
  By deciding against a DSL or other language/framework lock-ins for routing etc., you can implement authorization, connect, message routing, message handling and disconnect in your own favorite language. The only implementations required are the routing service and at least one destination for messages it points to.
* Message persistence and retries
  Messages are sent to a NATS/Jetstream stream which will buffer messages and only release if they are acknowledged. This will guarantee a at-least-once delivery for client->server messages.
