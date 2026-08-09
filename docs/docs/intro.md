# Introduction

**OxAPY** is a fast HTTP server for Python, built in Rust. Declare routes with decorators, return plain Python values, and let the framework handle the rest.

## Features

- Decorator-based routing with path parameters
- Middleware with per-router scoping
- Sessions, JWT authentication, CORS
- Template rendering, static files, file streaming
- Serializers with validation
- Async handlers
- Hot reload for development

## A taste of OxAPY

```python
from oxapy import Oxapy, Router, get


@get("/")
def hello(request):
    return "Hello, World!"


@get("/hello/{name}")
def greet(request, name):
    return {"message": f"Hello, {name}!"}


Oxapy(("127.0.0.1", 5555)).attach(Router().route(hello).route(greet)).run()
```

```bash
curl http://127.0.0.1:5555/
# Hello, World!

curl http://127.0.0.1:5555/hello/World
# {"message": "Hello, World!"}
```

## Next steps

- [Installation](./getting-started/installation) — install with pip
- [Quickstart](./getting-started/quickstart) — build your first app
- [Tutorial](./tutorial/notes-api) — a complete Notes API with SQLAlchemy, JWT, and serializers
