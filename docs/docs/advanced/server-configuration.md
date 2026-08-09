# Server Configuration

`Oxapy` is the heart of an OxAPY application. This page covers every configuration option.

## Constructing the server

```python
from oxapy import Oxapy

server = Oxapy(("127.0.0.1", 8000))
```

The single argument is a `(ip, port)` tuple.

## Fluent configuration

Every configuration method returns the server, so they chain naturally:

```python
from oxapy import Oxapy

server = (
    Oxapy(("127.0.0.1", 8000))
    .app_data(AppState())          # shared application data
    .attach(public_api)            # routers, checked in order
    .attach(admin_api)
    .cors(cors_config)             # automatic CORS handling
    .template(template)            # template engine
    .max_connections(1000)         # max concurrent connections (default 100)
    .channel_capacity(200)         # pending-request buffer (default 100)
    .wrap(global_wrapper)          # global response wrapper
)
```

## Reference

### app_data

`server.app_data(obj)` — store an object available to every handler via `request.app_data`. See the [Application State guide](../guides/app-state).

### attach

`server.attach(router)` — add a router. Routers are checked in order until a match is found. Returns the server.

### template

`server.template(template)` — enable template rendering. See the [Templates guide](../guides/templates).

### max_connections

`server.max_connections(n)` — maximum concurrent connections (default **100**). When the limit is reached, further connections wait for a slot.

### channel_capacity

`server.channel_capacity(n)` — how many pending requests can be buffered internally (default **100**). An advanced setting for tuning throughput under load.

### cors

`server.cors(cors)` — enable automatic CORS handling. The framework adds CORS headers to every response and handles preflight `OPTIONS` requests without hitting your handlers. CORS headers are applied after the `wrap()` wrapper. See the [CORS guide](../guides/cors).

### wrap

`server.wrap(callable)` — install a global wrapper invoked with `(request, response)` after every handler. Its return value is converted like a handler's return value. The pipeline order is: **handler → wrapper → CORS**. See the [Error Handling guide](../guides/error-handling).

### async_mode

`server.async_mode()` — enable async handlers. Returns the server, and `run()` then returns an awaitable. See the [Async Handlers guide](../guides/async-handlers).

### run

`server.run(reload=False, workers=None)` — start the blocking server. `reload=True` enables hot reload for development. `workers` sets the number of Tokio worker threads; when omitted the runtime decides.

```python
server.run()                    # default workers
server.run(reload=True)         # hot reload for development
server.run(workers=4)           # four worker threads
```

## Next steps

- [Deployment](./deployment) — running OxAPY in production
- [API Reference: Server](../api/server) — every method with signatures
