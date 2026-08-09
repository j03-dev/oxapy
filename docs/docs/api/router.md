# Router & Route

Routers register routes and their middleware; routes map a path and HTTP method to a handler.

## Route

### Constructor

```python
Route(path: str, method: str | None = None)
```

Creates a route for `path` and `method` (default `"GET"`). Calling it with a handler returns the route bound to that handler.

```python
from oxapy import Route

def handler(request):
    return "Hello, World!"

route = Route("/hello", "GET")
route = route(handler)
```

### Route helpers

`get`, `post`, `put`, `patch`, `delete`, `head`, and `options` create `Route` objects and work as decorators or as functions:

```python
from oxapy import get, post

# As a decorator
@get("/items/{item_id}")
def get_item(request, item_id: int):
    return {"item_id": item_id}

# As a function
route = post("/items", create_item)
```

All helpers share the signature `(path, handler=None)`.

## Router

### Constructor

```python
Router(base_path: str | None = None)
```

A `base_path` is prepended to every route registered on the router.

```python
router = Router("/api/v1")
```

### Methods

| Method | Description |
| --- | --- |
| `middleware(middleware)` | Add a middleware; applies to routes registered **after** it |
| `route(route)` | Register one route |
| `routes(routes)` | Register a list of routes |

All methods return the router, so calls chain:

```python
router = (
    Router("/api/v1")
    .route(get("/health", lambda _: "OK"))
    .middleware(auth_middleware)
    .routes([get_item, create_item])
)
```

### Middleware scoping

Middleware only wraps routes registered after it within the same router:

```python
router = (
    Router()
    .route(get("/health", lambda _: "OK"))   # no middleware
    .middleware(auth)
    .route(get("/dashboard", dashboard))      # auth
)
```

Use separate routers to isolate middleware groups. See the [Middleware guide](../guides/middleware).

### Attaching to the server

```python
server = HttpServer(("127.0.0.1", 8000))
server.attach(public_router).attach(admin_router)
```

Routers are checked in order until a matching route is found.

## Path parameters

- `{name}` — string parameter passed to the handler as a keyword argument
- `{name:int}` — integer parameter
- `{name:str}` — explicit string parameter
- `{*path}` — catch-all matching one or more segments

```python
@get("/users/{user_id:int}")
def get_user(request, user_id: int):
    return {"user_id": user_id}
```

## Related

- [Routing guide](../guides/routing) — examples and patterns
- [Middleware guide](../guides/middleware) — middleware scoping
- [Server](./server) — attaching routers to the server
