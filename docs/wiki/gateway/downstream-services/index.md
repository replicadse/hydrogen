# Downstream services

`hydrogen` will invoke a multitude of downstream services to process messages. Most of these are optional.


## Authorizer (optional)

### Request

```
{
  "type": "object",
  "properties": {
    "instance_id": {
      "type": "string"
    },
    "connection_id": {
      "type": "string"
    },
    "endpoint": {
      "type": "string"
    },
    "time": {
      "type": "string"
    },
    "headers": {
      "type": "array",
      "additionalItems": true,
      "items": [
        {
          "type": "array",
          "items": [
            {
              "type": "string"
            },
            {
              "type": "string"
            }
          ]
        }
      ]
    }
  },
  "required": [
    "instance_id",
    "connection_id",
    "time",
    "headers"
  ]
}
```

### Response

HTTP code 200 for success, other codes will make the connection abort due to an authorization error (401).

```
{
  "type": "object",
  "properties": {
    "context": {
      "type": "object",
      "additionalProperties": true
    }
  }
}
```

## Connect (optional)

### Request

```
{
  "type": "object",
  "properties": {
    "instance_id": {
      "type": "string"
    },
    "connection_id": {
      "type": "string"
    },
    "endpoint": {
      "type": "string"
    },
    "time": {
      "type": "string"
    }
  },
  "required": [
    "instance_id",
    "connection_id",
    "time"
  ]
}
```

### Response

HTTP code 200 for success, other codes will make the connection abort due to an internal error.

```
    *ignored*
```

## Disconnect (optional)

### Request

```
{
  "type": "object",
  "properties": {
    "instance_id": {
      "type": "string"
    },
    "connection_id": {
      "type": "string"
    },
    "endpoint": {
      "type": "string"
    },
    "time": {
      "type": "string"
    }
  },
  "required": [
    "instance_id",
    "connection_id",
    "time"
  ]
}
```

### Response

```
    *ignored*
```
