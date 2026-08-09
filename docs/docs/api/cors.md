# Cors

Configures Cross-Origin Resource Sharing headers for the server.

## Constructor

```python
Cors()
```

Creates a configuration with permissive defaults:

| Property | Default |
| --- | --- |
| `origins` | `["*"]` |
| `allow_credentials` | `True` |
| `max_age` | `86400` |

## Properties

| Property | Setter accepts | Description |
| --- | --- | --- |
| `origins` | `list[str]` | Allowed origins |
| `methods` | `list[str]` | Allowed HTTP methods |
| `headers` | `list[str]` | Allowed request headers |
| `allow_credentials` | `bool` | Allow cookies and authorization headers |
| `max_age` | `int` | Preflight cache duration in seconds |

## Example

```python
from oxapy import HttpServer, Cors

cors = Cors()
cors.origins = ["https://example.com", "https://app.example.com"]
cors.methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
cors.headers = ["Content-Type", "Authorization", "X-Custom-Header"]
cors.allow_credentials = True
cors.max_age = 3600

server = HttpServer(("127.0.0.1", 8000)).cors(cors)
server.run()
```

## Related

- [CORS guide](../guides/cors) — configuration walkthrough
- [Server](./server) — the `cors()` method
