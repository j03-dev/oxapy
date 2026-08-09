# CORS

Cross-Origin Resource Sharing lets browsers call your API from other origins. OxAPY handles CORS automatically — just configure `Cors` and attach it to the server.

## Basic configuration

```python
from oxapy import Oxapy, Router, Cors, get

cors = Cors()
cors.origins = ["https://example.com", "https://app.example.com"]

@get("/data")
def get_data(request):
    return {"message": "Hello from cross-origin!"}

server = Oxapy(("127.0.0.1", 8000))
server.cors(cors)
server.attach(Router().route(get_data))
server.run()
```

That's it. The framework adds CORS headers to every response and handles preflight `OPTIONS` requests automatically.

## How it works

When `server.cors(cors)` is configured, the server:

1. **Preflight requests** — Responds to `OPTIONS` requests with a `204 No Content` and the appropriate CORS headers, without hitting your handlers.
2. **Normal requests** — Applies CORS headers to the response after the handler chain (including any `wrap()` wrapper) completes.

The pipeline order is:

```
Handler → Wrapper (if any) → CORS headers applied
```

This guarantees CORS headers are always present on the final response, regardless of what your handler or wrapper does.

## Defaults

`Cors()` starts with permissive defaults:

| Setting | Default |
| --- | --- |
| `origins` | `["*"]` (all origins) |
| `allow_credentials` | `True` |
| `max_age` | `86400` (1 day) |

## All options

```python
cors = Cors()
cors.origins = ["https://example.com"]          # allowed origins
cors.methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
cors.headers = ["Content-Type", "Authorization", "X-Custom-Header"]
cors.allow_credentials = False                   # allow cookies/auth headers
cors.max_age = 3600                              # preflight cache in seconds
```

## Allowing credentials

Set `allow_credentials = True` (the default) when your frontend sends cookies or `Authorization` headers. Note that credentials are not combined with a wildcard origin in browsers, so list explicit origins.

## Next steps

- [Server Configuration](../advanced/server-configuration) — other server-level settings
- [API Reference: Cors](../api/cors) — every property and its default
