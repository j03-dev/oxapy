# Introduction

**OxAPY** is a fast HTTP server library for Python, implemented in Rust. It combines the ergonomics of a modern Python web framework with the performance and memory safety of a compiled core built on PyO3, `tokio`, and `hyper`.

OxAPY aims to be a next-generation, FastAPI-style framework: declare routes with decorators, return plain Python values, and let the framework handle routing, serialization, and HTTP details.

## Features

- Routing with path parameters (`/hello/{name}`), typed parameters, and catch-all routes
- Middleware support with per-router scoping
- Static file serving with path traversal protection
- Application state management shared across handlers
- Request and response handling, including form data and multipart file uploads
- Query string parsing
- Router base path prefixing for API versioning
- Template rendering (Tera/Jinja)
- Signed client-side sessions
- JWT authentication
- CORS configuration
- Serializers with validation
- Async handlers
- File streaming for large files
- Hot reload during development

## How it works

OxAPY is organized in two layers:

- A **Rust core** that runs the HTTP server on `hyper`/`tokio`, matches URLs with `matchit`, and serializes JSON with `orjson`.
- A **Python surface** (PyO3 bindings) where you define routes, middleware, and handlers using familiar Python code.

When a request arrives, the Rust runtime finds the matching route, walks the middleware chain, calls your Python handler, and converts its return value into a response.

## A taste of OxAPY

Save this as `app.py`:

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

Run it and try the endpoints:

```bash
python app.py
```

```bash
curl http://127.0.0.1:5555/
# Welcome to OxAPY!

curl http://127.0.0.1:5555/hello/World
# {"message": "Hello, World!"}
```

## Where to go next

- [Installation](./getting-started/installation) — install OxAPY with pip or build it from source
- [Quickstart](./getting-started/quickstart) — walk through your first app end to end
- [Build a Notes API](./tutorial/notes-api) — a complete, runnable tutorial: SQLAlchemy models, serializers, JWT auth, middleware, and async handlers
- [Routing guide](./guides/routing) — learn about path parameters, routers, and route registration
- [API Reference](./api/server) — the full class and function reference

## License

OxAPY is released under the [MIT License](https://github.com/j03-dev/oxapy/blob/main/LICENSE).
