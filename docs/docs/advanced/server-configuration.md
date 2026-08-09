# Server Configuration

The `HttpServer` class (and its `Oxapy` subclass) is the heart of an OxAPY application. This page covers every configuration option.

## Constructing the server

```python
from oxapy import HttpServer

server = HttpServer(("127.0.0.1", 8000))
```

The single argument is a `(ip, port)` tuple. `HttpServer` is the plain server; `Oxapy` adds hot reload.

## Fluent configuration

Every configuration method returns the server, so they chain naturally:

```python
from oxapy import HttpServer, Cors

server = (
    HttpServer(("127.0.0.1", 8000))
    .app_data(AppState())          # shared application data
    .attach(public_api)            # routers, checked in order
    .attach(admin_api)
    .template(template)            # template engine
    .cors(Cors())                  # CORS configuration
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

### cors

`server.cors(cors)` — apply CORS configuration. See the [CORS guide](../guides/cors).

### max_connections

`server.max_connections(n)` — maximum concurrent connections (default **100**). When the limit is reached, further connections wait for a slot.

### channel_capacity

`server.channel_capacity(n)` — how many pending requests can be buffered internally (default **100**). An advanced setting for tuning throughput under load.

### wrap

`server.wrap(callable)` — install a global wrapper invoked with `(request, response)` after every handler. Its return value is converted like a handler's return value. See the [Error Handling guide](../guides/error-handling).

### async_mode

`server.async_mode()` — enable async handlers. Returns the server, and `run()` then returns an awaitable. See the [Async Handlers guide](../guides/async-handlers).

### run

`server.run(workers=None)` — start the blocking server. `workers` sets the number of Tokio worker threads; when omitted the runtime decides.

```python
server.run()                    # default workers
server.run(workers=4)           # four worker threads
```

## The Oxapy subclass

`Oxapy` extends `HttpServer` with development hot reload:

```python
from oxapy import Oxapy

server = Oxapy(("127.0.0.1", 8000))
server.set_patterns(["*.py"])      # files to watch
server.set_watch_dir(".")          # directory to watch
server.run(reload=True)            # or run() for production
```

See the [Hot Reload guide](../guides/hot-reload).

## Next steps

- [Deployment](./deployment) — running OxAPY in production
- [API Reference: Server](../api/server) — every method with signatures
