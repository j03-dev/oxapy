# Cors

Configures Cross-Origin Resource Sharing headers. Use with `server.wrap()`.

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

## Methods

### apply_headers

```python
apply_headers(response: Response) -> None
```

Inserts CORS headers into the given response.

## Example

```python
from oxapy import Oxapy, Cors

cors = Cors()
cors.origins = ["https://example.com", "https://app.example.com"]


def cors_handler(request, response):
    cors.apply_headers(response)
    return response


server = Oxapy(("127.0.0.1", 8000))
server.wrap(cors_handler)
server.run()
```

## Related

- [CORS guide](../guides/cors) — configuration walkthrough
- [Server](./server) — the `wrap()` method
