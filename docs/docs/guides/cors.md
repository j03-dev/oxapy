# CORS

Cross-Origin Resource Sharing lets browsers call your API from other origins. Use `Cors` with `wrap()` to apply headers to every response.

## Defaults

`Cors()` starts with permissive defaults:

| Setting | Default |
| --- | --- |
| `origins` | `["*"]` (all origins) |
| `allow_credentials` | `True` |
| `max_age` | `86400` (1 day) |

## Basic configuration

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
