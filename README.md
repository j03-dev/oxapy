# OxAPY

<div align="center">
 <h4>
    <a href="https://github.com/j03-dev/oxapy/issues/">Report Bug</a>
 </h4>

<p>
  <b>OxAPY</b> is Python HTTP server library build in Rust - a fast, safe and featureementation.
</p>

<a href='https://github.com/j03-dev/oxapy/#'><img src='https://img.shields.io/badge/version-0.8.5-%23b7410e'/></a>
<a href="https://pepy.tech/projects/oxapy"><img src="https://static.pepy.tech/badge/oxapy" alt="PyPI Downloads"></a>

<p>
 <a href='https://pypi.org/project/oxapy/'> <img src='https://img.shields.io/pypi/v/oxapy?style=for-the-badge'/></a>
</p>

<p>
   <strong> Show your support</strong>  <em> by giving a star 🌟 if this project helped you! </em>
</p>

<p>
  <a href="https://bench-n9zz.onrender.com/bench/benchmark_rps.png"><img src="https://bench-n9zz.onrender.com/bench/benchmark_rps.png"/></a>
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

import asyncio

@get("/")
async def home(request):
    # Asynchronous operations are allowed here
    data = await fetch_data_from_database()  
    return "Hello, World!"

async def main():
    await (
        HttpServer(("127.0.0.1", 8000))
        .attach(
            Router().route(home)
        )
        .async_mode()
        .run()
    )

if __name__ == "__main__":
    asyncio.run(main())
```

## Middleware

OxAPY offers two paradigms for organizing middleware. You can use one or combine both.

### 1. Sequence Paradigm (same router)

Middleware only applies to routes registered **after** it within the same router. Routes before it get no middleware.

```python
# Simple: one middleware layer
Router()
.route(get("/health", lambda _: "OK"))       # no middleware
.middleware(auth)
.route(get("/dashboard", dashboard))          # auth only
.route(get("/account", account))              # auth only
```

```python
# Multiple layers: each middleware applies to everything after it
Router()
.route(get("/health", lambda _: "OK"))        # no middleware
.route(static_file())                         # no middleware
.middleware(session)
.route(get("/login", login))                  # session
.route(get("/register", register))            # session
.middleware(db_session)
.route(get("/search", search))                # session + db_session
.route(get("/profile", profile))              # session + db_session
.middleware(protect_page)
.route(get("/admin", admin))                  # session + db_session + protect_page
```

### 2. Multi-Router Paradigm (separate routers)

Each router has its own independent middleware stack. Routers are checked in order until a match is found. Use this when groups share no middleware.

```python
# Simple: two isolated groups
HttpServer(("127.0.0.1", 5555))
.attach(
    Router()
    .route(get("/health", lambda _: "OK"))
    .route(static_file())
)
.attach(
    Router()
    .middleware(auth)
    .route(get("/dashboard", dashboard))
    .route(get("/account", account))
)
```

```python
# Multiple isolated groups with different middleware stacks
HttpServer(("127.0.0.1", 5555))
.attach(
    Router()
    .route(static_file())
    .route(get("/health", lambda _: "Good!"))
)
.attach(
    Router()
    .middleware(session)
    .middleware(db_session)
    .routes([login_user, register_user, show_login_page])
)
.attach(
    Router()
    .middleware(session)
    .middleware(db_session)
    .middleware(protect_page)
    .routes([show_dashboard, show_account, logout_user])
)
```

### 3. Combined (both paradigms)

Use sequence layering inside a router alongside separate routers.

```python
HttpServer(("127.0.0.1", 5555))
.attach(
    Router()
    .route(get("/health", lambda _: "OK"))         # no middleware
    .middleware(rate_limit)
    .route(get("/login", login))                    # rate_limit
    .route(get("/register", register))              # rate_limit
)
.attach(
    Router()
    .middleware(session)
    .middleware(db_session)
    .route(get("/dashboard", dashboard))            # session + db_session
    .middleware(protect_page)
    .route(get("/admin", admin))                    # session + db_session + protect_page
)
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
