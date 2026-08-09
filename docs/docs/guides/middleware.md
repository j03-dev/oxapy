# Middleware

Middleware runs between the router and your handlers. A middleware function receives the request and a `next` callable; it can inspect the request, short-circuit the chain, or decorate the final response.

## Writing a middleware

A middleware is a plain function with the signature `(request, next, **kw)`:

```python
def auth_middleware(request, next, **kw):
    if "authorization" not in request.headers:
        return Status.UNAUTHORIZED
    request.user_name = "John Doe"
    return next(request, **kw)
```

- Return a response or `Status` directly to stop the chain early (short-circuit).
- Call `next(request, **kw)` to pass control to the next middleware or the handler.
- Set attributes on `request` before calling `next`; handlers can read them afterwards.

Register middleware with `router.middleware(...)`:

```python
from oxapy import Oxapy, Router, Status, get


def auth_middleware(request, next, **kw):
    if "authorization" not in request.headers:
        return Status.UNAUTHORIZED
    return next(request, **kw)


@get("/protected")
def protected(request):
    return {"message": "You are authenticated"}


def main():
    router = Router().middleware(auth_middleware).route(protected)
    Oxapy(("127.0.0.1", 5555)).attach(router).run()


if __name__ == "__main__":
    main()
```

## Scoping: how middleware applies

Middleware only applies to routes registered **after** it within the same router. Routes registered before it are not affected.

```python
# Sequence paradigm: each middleware applies to everything registered after it
(
    Router()
    .route(get("/health", lambda _: "OK"))        # no middleware
    .route(static_file())                         # no middleware
    .middleware(session)
    .route(get("/login", login))                  # session
    .route(get("/register", register))            # session
    .middleware(db_session)
    .route(get("/search", search))                # session + db_session
    .middleware(protect_page)
    .route(get("/admin", admin))                  # session + db_session + protect_page
)
```

When routes share no middleware, use separate `Router` instances. Routers are checked in order until a match is found, so each group keeps its own middleware stack.

```python
(
    Oxapy(("127.0.0.1", 5555))
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
)
```

:::tip

Think of middleware as layers: within one router it is a sequence where each layer wraps everything registered after it; across routers it is a set of independent groups.

:::

## Middleware from the standard library

`functools.partial` works out of the box, which is how the built-in [Session](../guides/sessions) middleware is produced:

```python
from oxapy import Session

session = Session(b"my-secret-key")  # a partially-applied middleware
router = Router().middleware(session).route(login)
```

## Order of execution

Middleware functions run in the order they are added. The first registered middleware is the outermost layer: it runs first on the way in and last on the way out.

## Production patterns

### Per-request resources (database sessions)

Open a resource, attach it to the request, and close it after the handler runs. Handlers read it via `request.db`:

```python
from typing import Callable
from oxapy import Request, Response

Next = Callable[[Request], Response]


def db(req: Request, next: Next, **kwargs) -> Response:
    with DB() as _db:  # context manager opens and closes the session
        req.db = _db
        return next(req, **kwargs)
```

```python
@get("/users/{user_id}")
def get_user(request, user_id: int):
    user = user_srvs.retrieve(request.db, user_id)   # request.db from middleware
    return UserSerializer(instance=user).data
```

### Short-circuiting with a redirect

Instead of an error response, redirect unauthenticated users to the login page:

```python
def protect_page(req: Request, next: Next, **kwargs) -> Response:
    session = req.session
    if session.get("is_auth"):
        req.user_id = session.get("user_id")
        return next(req, **kwargs)
    return Redirect("/login")
```

### Layering different middleware over different route groups

Chain `.middleware(...)` with `.routes([...])` to give each group its own stack:

```python
router = (
    Router("/api")
    .middleware(db)
    .routes([signup, signin])          # db only
    .middleware(jwt)
    .routes([me, create_group, loan])  # db + jwt
)
```

Every middleware applies to the routes registered after it, so the second group inherits `db` and adds `jwt`.

## Next steps

- [Sessions](./sessions) — the built-in signed-cookie session middleware
- [Requests](./requests) — what you can read from the request object
- [API Reference: Router](../api/router) — `middleware()`, `route()`, and `routes()`
