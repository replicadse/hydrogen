# Downstream services

`spoderman` will invoke a multitude of downstream services to process messages. Most of these are optional.


## Rules engine (required)

### Request

```
{
  "$schema": "http://json-schema.org/draft-04/schema#",
  "type": "object",
  "properties": {
    "instance_id": {
      "type": "string"
    },
    "connection_id": {
      "type": "string"
    },
    "time": {
      "type": "string"
    },
    "message": {
      "type": "string"
    },
    "context": {
      "type": "object",
      "additionalProperties": true
    }
  },
  "required": [
    "instance_id",
    "connection_id",
    "time",
    "context",
    "message"
  ]
}
```

### Response

HTTP code 200 for success, other codes will make the message not being consumed.

```
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string"
    },
    "headers": {
      "type": "object",
      "additionalProperties": true
    }
  },
  "required": [
    "endpoint",
    "headers"
  ]
}
```

## Message destination (given by rules engine)

### Request

```
{
  "$schema": "http://json-schema.org/draft-04/schema#",
  "type": "object",
  "properties": {
    "instance_id": {
      "type": "string"
    },
    "connection_id": {
      "type": "string"
    },
    "time": {
      "type": "string"
    },
    "message": {
      "type": "string"
    },
    "context": {
      "type": "object",
      "additionalProperties": true
    }
  },
  "required": [
    "instance_id",
    "connection_id",
    "time",
    "context",
    "message"
  ]
}
```

### Response

HTTP code 200 for success, other codes will make the message not being consumed.

```
    *ignored*
```
