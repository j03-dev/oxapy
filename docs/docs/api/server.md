# Oxapy

The server is the main entry point of an OxAPY application. It manages routers, middleware, templates, sessions, and the runtime itself.

## Constructor

```python
Oxapy(addr: tuple[str, int])
```

Creates a server bound to the given address.

```python
from oxapy import Oxapy

server = Oxapy(("127.0.0.1", 8000))
```

## Methods

| Method | Description |
| --- | --- |
| `app_data(app_data)` | Store application-wide data; readable in handlers via `request.app_data` |
| `attach(router)` | Attach a router; routers are checked in order until a match |
| `cors(cors)` | Enable automatic CORS handling and preflight responses |
| `template(template)` | Enable template rendering |
| `max_connections(max_connections)` | Max concurrent connections (default `100`) |
| `channel_capacity(channel_capacity)` | Internal pending-request buffer (default `100`) |
| `wrap(wrapper)` | Install a global `(request, response)` wrapper for response transformation |
| `async_mode()` | Enable async handlers; `run()` becomes awaitable |
| `set_patterns(patterns)` | Glob patterns the reload watcher reacts to (default `["*.py"]`) |
| `set_watch_dir(dir)` | Directory tree the reload watcher observes (default `"."`) |
| `run(reload=False, processes=None, workers=None)` | Start the server, optionally as a pool of processes |

All configuration methods return the server for chaining:

```python
from oxapy import Oxapy

server = (
    Oxapy(("127.0.0.1", 8000))
    .max_connections(1000)
    .run()
)
```

## run

```python
run(reload: bool = False, processes: int | None = None, workers: int | None = None) -> Any
```

Starts the server and blocks until interrupted. Only available on `Oxapy`; `HttpServer.run()` takes `workers` alone.

| Argument | Default | Description |
| --- | --- | --- |
| `reload` | `False` | Watch for file changes and restart the worker pool (development only) |
| `processes` | `None` → `1` | Number of **OS processes** sharing the port via `SO_REUSEPORT` (Unix only) |
| `workers` | `None` | Number of **Tokio worker threads** per process; when omitted the runtime decides |

```python
server.run()                        # one process, runtime-chosen thread count
server.run(workers=4)               # one process, four Tokio threads
server.run(processes=4)             # four processes on the same port
server.run(processes=4, workers=2)  # four processes, two threads each
server.run(reload=True)             # hot reload during development
```

Values of `processes` that are `None`, `0`, or negative are treated as `1`.

:::note `reload` and `processes` share one supervisor

Both arguments are handled by the same Python supervisor. Any spawn of more than one process — `reload=True`, `processes > 1`, or both — requires an `if __name__ == "__main__":` guard in your entry point, because workers are created by re-executing your script. `processes > 1` is Unix only, and there is no graceful drain of in-flight requests on restart. See the [Multiprocess guide](../guides/multiprocess) for the full picture.

:::

## cors

```python
cors(cors: Cors) -> Oxapy
```

Enables automatic CORS handling. The framework adds CORS headers to every response and handles preflight `OPTIONS` requests without hitting your handlers.

```python
from oxapy import Cors

cors = Cors()
cors.origins = ["https://example.com"]

server.cors(cors)
```

CORS headers are applied **after** the `wrap()` wrapper, so they are always present on the final response.

## wrap

```python
wrap(wrapper) -> Oxapy
```

The wrapper is called with `(request, response)` after the handler chain completes and its return value is converted to a response. The pipeline order is: **handler → wrapper → CORS headers**.

```python
def global_middleware(request, response):
    if response.status == Status.NOT_FOUND:
        return Response("<h1>Page Not Found</h1>", content_type="text/html")
    return response

server.wrap(global_middleware)
```

## Hot reload

`Oxapy` supports hot reload for development:

```python
from oxapy import Oxapy

server = (
    Oxapy(("127.0.0.1", 5555))
    .set_patterns(["*.py", "*.html"])
    .set_watch_dir("src")
    .attach(router)
    .run(reload=True)
)
```

With `reload=True`, the instance acts as a supervisor that spawns a pool of worker processes and restarts the whole pool when a watched file changes. See the [Hot Reload guide](../guides/hot-reload).

## Multiprocess serving

`processes=N` runs the server from N OS processes sharing the listening port via `SO_REUSEPORT`:

```python
Oxapy(("127.0.0.1", 5555)).attach(router).run(processes=4)
```

A crashed worker is respawned automatically. See the [Multiprocess guide](../guides/multiprocess) — including the Unix-only limitation and the `__main__` guard requirement.

## Examples

### Basic app

```python
from oxapy import Oxapy, Router, get

@get("/")
def home(request):
    return "Hello, World!"

server = Oxapy(("127.0.0.1", 8000)).attach(Router().route(home))
server.run()
```

### Async app

```python
import asyncio
from oxapy import Oxapy, Router, get

@get("/")
async def home(request):
    return "Hello, World!"

async def main():
    await Oxapy(("127.0.0.1", 8000)).attach(Router().route(home)).async_mode().run()

asyncio.run(main())
```

## Related

- [Router & Route](./router) — the `Router` class
- [Server Configuration](../advanced/server-configuration) — configuration guide
- [Static Files API](./static-files) — `static_file`, `send_file`, `secure_join`
