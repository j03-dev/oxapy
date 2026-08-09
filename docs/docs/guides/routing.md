# Routing

OxAPY routes map a URL path and an HTTP method to a Python handler. Routes are created with decorators such as `@get` and `@post`, then registered with a `Router`, which is attached to the server.

## Route decorators

Every HTTP method has a matching decorator: `@get`, `@post`, `@put`, `@patch`, `@delete`, `@head`, and `@options`.

```python
from oxapy import Oxapy, Router, get, post, put, delete


@get("/items")
def list_items(request):
    return {"items": []}


@post("/items")
def create_item(request):
    return {"status": "created"}


@put("/items/{item_id}")
def update_item(request, item_id: int):
    return {"status": "updated", "item_id": item_id}


@delete("/items/{item_id}")
def delete_item(request, item_id: int):
    return {"status": "deleted", "item_id": item_id}


def main():
    (
        Oxapy(("127.0.0.1", 5555))
        .attach(
            Router().routes([list_items, create_item, update_item, delete_item])
        )
        .run()
    )


if __name__ == "__main__":
    main()
```

The decorators are also plain functions, so you can pass a handler directly:

```python
from oxapy import Router, get

def hello_handler(request):
    return "Hello World!"

route = get("/hello", hello_handler)
router = Router().route(route)
```

## Registering routes

Use `.route()` for a single route and `.routes()` for a list. Both return the router, so calls can be chained.

```python
router = (
    Router()
    .route(get("/health", lambda _: "OK"))
    .routes([list_items, create_item])
)
```

## Path parameters

Path parameters are declared with curly braces and passed to the handler as keyword arguments.

```python
from oxapy import Router, get


@get("/hello/{name}")
def hello(request, name):
    return f"Hello, {name}!"


@get("/users/{user_id}")
def get_user(request, user_id: int):
    return {"user_id": user_id, "name": f"User {user_id}"}
```

### Typed parameters

Two types are built in: `{name:str}` and `{name:int}`. Using `{user_id:int}` gives you an `int` in the handler without manual conversion.

```python
@get("/users/{user_id:int}")
def get_user(request, user_id: int):
    return {"user_id": user_id}
```

### Catch-all parameters

Use `{*path}` to match one or more path segments. This is handy for static files and downloads.

```python
@get("/files/{*path}")
def serve_file(request, path):
    return f"Requested file: {path}"
```

A request to `/files/docs/readme.txt` calls the handler with `path="docs/readme.txt"`.

## Router base path

A `Router` can be created with a `base_path` that is prepended to every route registered on it. This is the recommended way to version an API.

```python
from oxapy import Oxapy, Router, get


@get("/users")
def get_users(request):
    return [{"id": 1, "name": "user1"}]


def main():
    (
        Oxapy(("127.0.0.1", 5555))
        .attach(Router("/api/v1").route(get_users))
        .run()
    )


if __name__ == "__main__":
    main()
```

The endpoint is now served at `http://127.0.0.1:5555/api/v1/users`.

## Multiple routers

You can attach any number of routers to a server. They are checked in order until a matching route is found.

```python
public_api = Router("/api").route(get("/health", lambda _: "OK"))
admin_api = Router("/admin").middleware(auth_middleware).route(get("/stats", stats))

Oxapy(("127.0.0.1", 5555)).attach(public_api).attach(admin_api).run()
```

Using separate routers is also how you isolate middleware to specific groups. See the [Middleware guide](./middleware).

## Unmatched routes

When no route matches the request, the server responds with `404 Not Found`. Note that a request whose method does not match any route on the path is also answered with 404 (rather than 405) in the current implementation.

## Next steps

- [Requests](./requests) — read headers, query strings, JSON bodies, forms, and uploads
- [Responses](./responses) — return values, status codes, and custom headers
- [Middleware](./middleware) — process requests before handlers run
