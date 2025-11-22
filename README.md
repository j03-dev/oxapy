# OxAPY

<div align="center">
 <h4>
    <a href="https://github.com/j03-dev/oxapy/issues/">Report Bug</a>
 </h4>

<p>
  <b>OxAPY</b> is Python HTTP server library build in Rust - a fast, safe and feature-rich HTTP server implementation.
</p>

<a href='https://github.com/j03-dev/oxapy/#'><img src='https://img.shields.io/badge/version-0.6.9-%23b7410e'/></a>
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

router = Router()


@get("/")
def welcome(request):
    return Response("Welcome to OxAPY!", content_type="text/plain")


@get("/hello/{name}")
def hello(request, name):
    return Response({"message": f"Hello, {name}!"})

router.routes([welcome, hello])

app = HttpServer(("127.0.0.1", 5555))
app.attach(router)

if __name__ == "__main__":
    app.run()
```

## Async Example

```python
from oxapy import HttpServer, Router, get

router = Router()


@get("/")
async def home(request):
    # Asynchronous operations are allowed here
    data = await fetch_data_from_database()  # type: ignore
    return "Hello, World!"

router.route(home)

HttpServer(("127.0.0.1", 8000)).attach(router).async_mode().run()
```

## Middleware Example

OxAPY's middleware system is designed to be flexible and powerful. Middleware is applied on a per-layer basis. When you add middleware, it "closes" the current layer of routes and applies to all routes within that layer. Any subsequent routes are added to a new, clean layer. This allows you to build complex routing structures with different middleware for different groups of routes.

### Best Practices

- **Order Matters**: Middleware is executed in the order it is defined. Ensure that middleware with dependencies is ordered correctly.
- **Layering**: Use multiple middleware layers to separate concerns. For example, you can have one layer for authentication and another for logging.
- **Clarity**: Be mindful of the routes that fall under each middleware. The middleware will apply to all routes that are defined before it in the same layer.

```python
from oxapy import Status, Router, get

def log_middleware(request, next, **kwargs):
    print(f"Request: {request.method} {request.path}")
    return next(request, **kwargs)

def auth_middleware(request, next, **kwargs):
    if "authorization" not in request.headers:
        return Status.UNAUTHORIZED
    return next(request, **kwargs)

router = Router()

# Define a public route
router.route(get("/public", lambda req: "Public"))

# Add logging middleware. This closes the first layer.
# log_middleware will apply to the "/public" route.
router.middleware(log_middleware)

# Define a protected route in a new layer
router.route(get("/protected", lambda req: "Protected"))

# Add auth middleware. This closes the second layer.
# auth_middleware will apply to the "/protected" route, but not the "/public" route.
router.middleware(auth_middleware)
```

## Static Files

```python
from oxapy import Router, static_file

router = Router()
router.route(static_file("/static", "./static"))
# Serves files from ./static directory at /static URL path
```

## Application State

```python
from oxapy import HttpServer, Router, get


class AppState:
    def __init__(self):
        self.counter = 0


router = Router()


@get("/count")
def handler(request):
    app_data = request.app_data
    app_data.counter += 1
    return {"count": app_data.counter}

router.route(handler)

HttpServer(("127.0.0.1", 5555)).app_data(AppState()).attach(router).run()
```

## Router with Base Path

You can set a base path for a router, which will be prepended to all routes defined in it. This is useful for versioning APIs.

```python
from oxapy import HttpServer, Router, get

# All routes in this router will be prefixed with /api/v1
router = Router("/api/v1")

@get("/users")
def get_users(request):
    return [{"id": 1, "name": "user1"}]

router.route(get_users)

app = HttpServer(("127.0.0.1", 5555))
app.attach(router)

if __name__ == "__main__":
    app.run()

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
