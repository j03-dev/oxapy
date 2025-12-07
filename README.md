# OxAPY

<div align="center">
 <h4>
    <a href="https://github.com/j03-dev/oxapy/issues/">Report Bug</a>
 </h4>

<p>
  <b>OxAPY</b> is Python HTTP server library build in Rust - a fast, safe and feature-rich HTTP server implementation.
</p>

<a href='https://github.com/j03-dev/oxapy/#'><img src='https://img.shields.io/badge/version-0.7.6-%23b7410e'/></a>
<a href="https://pepy.tech/projects/oxapy"><img src="https://static.pepy.tech/badge/oxapy" alt="PyPI Downloads"></a>

<p>
 <a href='https://pypi.org/project/oxapy/'> <img src='https://img.shields.io/pypi/v/oxapy?style=for-the-badge'/></a>
</p>

<p>
   <strong> Show your support</strong>  <em> by giving a star 🌟 if this project helped you! </em>
</p>

<p>
  <a href="https://github.com/j03-dev/bench"><img src="https://bench-n9zz.onrender.com/bench"/></a>
</p>
</div>

## Features

- Routing with path parameters
- Middleware support
- Static file serving
- Application state management
- Request/Response handling
- Query string parsing
- Router base path prefixing

## Basic Example

```python
from oxapy import HttpServer, Router, Status, Response, get

@get("/")
def welcome(request):
    return Response("Welcome to OxAPY!", content_type="text/plain")

@get("/hello/{name}")
def hello(request, name):
    return Response({"message": f"Hello, {name}!"})

def main():
    (
        HttpServer(("127.0.0.1", 5555))
        .attach(
            Router()
            .route(welcome)
            .route(hello)
        )
        .run()
    )

if __name__ == "__main__":
    main()
```

## Async Example

```python
from oxapy import HttpServer, Router, get

@get("/")
async def home(request):
    # Asynchronous operations are allowed here
    data = await fetch_data_from_database()  # type: ignore
    return "Hello, World!"

def main():
    (
        HttpServer(("127.0.0.1", 8000))
        .attach(
            Router().route(home)
        )
        .async_mode()
        .run()
    )

if __name__ == "__main__":
    main()
```

## Middleware Example

OxAPY's middleware system is designed to be flexible and powerful. Middleware is applied to all routes within the same **scope**. You can use the `.scope()` method to create new scopes, allowing you to group routes with different middleware. This allows for building complex routing structures where different sets of middleware apply to different groups of routes.

### Best Practices

- **Order Matters**: Middleware is executed in the order it is defined within a scope.
- **Scoping**: Use `.scope()` to create logical separation between groups of routes and their middleware. For example, you can have one scope for public endpoints and another for authenticated endpoints.
- **Clarity**: Be mindful that middleware applies to all routes defined in the current scope, both before and after the middleware is added.

```python
from oxapy import Status, Router, get, HttpServer

def log_middleware(request, next, **kwargs):
    print(f"Request: {request.method} {request.uri}")
    return next(request, **kwargs)

def auth_middleware(request, next, **kwargs):
    if "authorization" not in request.headers:
        return Status.UNAUTHORIZED
    return next(request, **kwargs)

@get("/public")
def public(request):
    return "This is a public route."

@get("/protected")
def protected(request):
    return "This is a protected route."

def main():
    (
        HttpServer(("127.0.0.1", 5555))
        .attach(
            Router()
            # First scope: public routes with logging
            .route(public)
            .middleware(log_middleware)

            # Second scope: protected routes with logging and authentication
            .scope()
            .route(protected)
            .middleware(log_middleware)
            .middleware(auth_middleware)
        )
        .run()
    )

if __name__ == "__main__":
    main()
```

## Static Files

```python
from oxapy import HttpServer, Router, static_file

def main():
    (
        HttpServer(("127.0.0.1", 5555))
        .attach(
            Router().route(static_file("/static", "./static"))
        )
        .run()
    )

if __name__ == "__main__":
    main()
```

## Application State

```python
from oxapy import HttpServer, Router, get

class AppState:
    def __init__(self):
        self.counter = 0

@get("/count")
def handler(request):
    app_data = request.app_data
    app_data.counter += 1
    return {"count": app_data.counter}

def main():
    (
        HttpServer(("127.0.0.1", 5555))
        .app_data(AppState())
        .attach(
            Router().route(handler)
        )
        .run()
    )

if __name__ == "__main__":
    main()
```

## Router with Base Path

You can set a base path for a router, which will be prepended to all routes defined in it. This is useful for versioning APIs.

```python
from oxapy import HttpServer, Router, get

@get("/users")
def get_users(request):
    return [{"id": 1, "name": "user1"}]

def main():
    (
        HttpServer(("127.0.0.1", 5555))
        .attach(
            Router("/api/v1").route(get_users)
        )
        .run()
    )

if __name__ == "__main__":
    main()

# You can now access the endpoint at http://127.0.0.1:5555/api/v1/users
```

Todo:

- [x] Handler
- [x] HttpResponse
- [x] Routing
- [x] use tokio::net::Listener
- [x] middleware
- [x] app data
- [x] pass request in handler
- [x] serve static file
- [x] templating
- [x] query uri
- [ ] security submodule
    - [x] jwt
    - [ ] bcrypt
- [ ] websocket
