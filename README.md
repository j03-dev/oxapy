# OxAPY

<div align="center">
 <h4>
    <a href="https://github.com/j03-dev/oxapy/issues/">Report Bug</a>
 </h4>

<p>
  <b>OxAPY</b> is Python HTTP server library build in Rust - a fast, safe and feature-rich HTTP server implementation.
</p>

<a href='https://github.com/j03-dev/oxapy/#'><img src='https://img.shields.io/badge/version-0.5.9-%23b7410e'/></a>
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

## Basic Example

```python
from oxapy import HttpServer, Router, Status, Response

router = Router()

@router.get("/")
def welcome(request):
    return Response("Welcome to OxAPY!", content_type="text/plain")

@router.get("/hello/{name}")
def hello(request, name):
    return Response({"message": f"Hello, {name}!"})


app = HttpServer(("127.0.0.1", 5555))
app.attach(router)

if __name__ == "__main__":
    app.run()
```

## Middleware Example

```python
def auth_middleware(request, next, **kwargs):
    if "authorization" not in request.headers:
        return Status.UNAUTHORIZED
    return next(request, **kwargs)

router = Router()
router.middleware(auth_middleware)

@router.get("/protected")
def protected(request):
    return "This is protected!"
```

## Static Files

```python
router = Router()
router.route(static_file("./static", "static"))
# Serves files from ./static directory at /static URL path
```

## Application State

```python
class AppState:
    def __init__(self):
        self.counter = 0

app = HttpServer(("127.0.0.1", 5555))
app.app_data(AppState())

router = Router()

@router.get("/count")
def handler(request):
    app_data = request.app_data
    app_data.counter += 1
    return {"count": app_data.counter}

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
