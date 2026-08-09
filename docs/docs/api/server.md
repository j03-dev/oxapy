# HttpServer / Oxapy

The server is the main entry point of an OxAPY application. It manages routers, middleware, templates, sessions, CORS, and the runtime itself.

## HttpServer

### Constructor

```python
HttpServer(addr: tuple[str, int])
```

Creates a server bound to the given address.

```python
from oxapy import HttpServer

server = HttpServer(("127.0.0.1", 8000))
```

### Methods

| Method | Description |
| --- | --- |
| `app_data(app_data)` | Store application-wide data; readable in handlers via `request.app_data` |
| `attach(router)` | Attach a router; routers are checked in order until a match |
| `template(template)` | Enable template rendering |
| `cors(cors)` | Apply a CORS configuration |
| `max_connections(max_connections)` | Max concurrent connections (default `100`) |
| `channel_capacity(channel_capacity)` | Internal pending-request buffer (default `100`) |
| `wrap(wrapper)` | Install a global `(request, response)` wrapper |
| `async_mode()` | Enable async handlers; `run()` becomes awaitable |
| `run(workers=None)` | Start the blocking server |

All configuration methods return the server for chaining:

```python
from oxapy import HttpServer, Cors

server = (
    HttpServer(("127.0.0.1", 8000))
    .max_connections(1000)
    .cors(Cors())
    .run()
)
```

### run

```python
run(workers: int | None = None) -> Any
```

Starts the server and blocks until interrupted. `workers` sets the number of Tokio worker threads; when omitted the runtime decides.

### wrap

The wrapper is called with `(request, response)` after the handler chain and its return value is converted to a response:

```python
def global_middleware(request, response):
    if response.status == Status.NOT_FOUND:
        return Response("<h1>Page Not Found</h1>", content_type="text/html")
    return response

server.wrap(global_middleware)
```

## Oxapy

`Oxapy` subclasses `HttpServer` and adds hot reload for development.

```python
class Oxapy(HttpServer):
    def run(self, reload: bool = False, workers: int | None = None) -> Any
    def set_patterns(self, p: list[str]) -> Oxapy
    def set_watch_dir(self, dir: str) -> Oxapy
```

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

With `reload=True`, the instance acts as a supervisor that restarts a worker process when watched files change. See the [Hot Reload guide](../guides/hot-reload).

## Examples

### Basic app

```python
from oxapy import HttpServer, Router, get

@get("/")
def home(request):
    return "Hello, World!"

server = HttpServer(("127.0.0.1", 8000)).attach(Router().route(home))
server.run()
```

### Async app

```python
import asyncio
from oxapy import HttpServer, Router, get

@get("/")
async def home(request):
    return "Hello, World!"

async def main():
    await HttpServer(("127.0.0.1", 8000)).attach(Router().route(home)).async_mode().run()

asyncio.run(main())
```

## Related

- [Router & Route](./router) — the `Router` class
- [Server Configuration](../advanced/server-configuration) — configuration guide
