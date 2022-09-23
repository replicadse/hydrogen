# Endpoints

## `WS @ /ws/{}`

This is the primary socket endpoint clients need connect to. Endpoints are specified in the config file. It will trigger the connection pipeline before and during connect and trigger a disconnect event on client disconnect. \
Messages are sent through the open connections to this endpoint both from client to server and vice versa. \
When connecting, every established connection gets a unique `connection_id` assigned that is also transported to every downstream service which is invoked at any point (since the connection was not permitted yet at that point in time). Keep in mind that this id is given per connection and one client could have more than one connection open.

## `HTTP/GET @ /health`

A basic health check endpoint. Will return `code 200` and a static JSON formatted response body.

## `HTTP/POST @ /connections/$connection_id/_send`

This endpoint is used in order to have a message sent from the backend to a connected client. The request body will be transmitted as text.

## `HTTP/POST @ /connections/$connection_id/_disconnect`

Forces a disconnect for the given connection.

## `HTTP/POST @ /connections/_broadcast`

Broadcasts a message to all connections. \
Optional query params are `endpoints[]=`, can be set multiple times. If specified, only connections to the specified endpoints will receive the broadcast message. Example: `$BASE_URL/connections/_broadcast?endpoints%5B%5D=%2Ftest&endpoints%5B%5D=%2F` will send broadcast to endpoint `/` and `/test`.