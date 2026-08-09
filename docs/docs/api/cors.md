# Cors

Configures Cross-Origin Resource Sharing headers. Attach it with `server.cors(cors)` to enable automatic CORS handling.

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
from oxapy import Oxapy, Cors, Router, get

cors = Cors()
cors.origins = ["https://example.com", "https://app.example.com"]

@get("/api/data")
def get_data(request):
    return {"message": "Hello"}

server = Oxapy(("127.0.0.1", 8000))
server.cors(cors)
server.attach(Router().route(get_data))
server.run()
```

## Related

- [CORS guide](../guides/cors) — configuration walkthrough
- [Server](./server) — the `cors()` method
