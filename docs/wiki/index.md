# spoderman

`spoderman` is a kubernetes native API gateway for websocket connections that indirects messages into HTTP requests.

## Features

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

## Roadmap to v1

* Prometheus metrics support
* Documentation / Wiki / Manual

## Post v1

* UI & Dashboard
* Administration UI
* Redis channel alternatives (?)
