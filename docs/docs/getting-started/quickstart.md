# Quickstart

## Create your first app

Save the following as `app.py`:

```python
from oxapy import Oxapy, Router, Response, get


@get("/")
def welcome(request):
    return Response("Welcome to OxAPY!", content_type="text/plain")


@get("/hello/{name}")
def hello(request, name):
    return Response({"message": f"Hello, {name}!"})


def main():
    (
        Oxapy(("127.0.0.1", 5555))
        .attach(Router().route(welcome).route(hello))
        .run()
    )


if __name__ == "__main__":
    main()
```

## Run it

```bash
python app.py
```

The server blocks until you stop it with `Ctrl+C`. In another terminal, try the endpoints:

```bash
curl http://127.0.0.1:5555/
# Welcome to OxAPY!

curl http://127.0.0.1:5555/hello/World
# {"message": "Hello, World!"}
```

:::tip
During development, use `reload=True` to automatically restart the server when files change:

```python
server.run(reload=True)
```

:::

## How it works

- `@get("/")` and `@get("/hello/{name}")` decorate handlers and produce `Route` objects.
- `Router().route(...)` registers each route; `{name}` is a path parameter passed as a keyword argument to the handler.
- `.attach(router)` mounts the router on the server.
- `.run()` starts the blocking server on `127.0.0.1:5555`.

:::note

Handlers receive the `Request` object as their first argument. Returning a plain `dict` produces a JSON response; returning a `str` produces plain text. See the [Responses guide](../guides/responses) for the full list of supported return types.

:::

## What's next

- [Build a Notes API](../tutorial/notes-api) — a complete, runnable tutorial with SQLAlchemy, JWT, and serializers
- [Routing](../guides/routing) — path parameters, typed parameters, catch-all routes, and router base paths
- [Requests](../guides/requests) — reading headers, JSON bodies, forms, and files
- [Middleware](../guides/middleware) — processing requests before they reach your handlers
- [API Reference](../api/server) — the complete reference for every class and function
