# Sessions

OxAPY sessions are **client-side and signed**: the server stores session data in a cookie, so no session table is needed. The cookie payload is signed with HMAC-SHA256 using your secret, which prevents clients from tampering with it.

## Enabling sessions

Create a session middleware with `Session(secret, max_age)` and register it on a router:

```python
from oxapy import Oxapy, Router, Session, get


@get("/")
def home(request):
    request.session["visited"] = True
    return {"session": dict(request.session)}


def main():
    session = Session(b"my-secret-key")  # bytes secret
    (
        Oxapy(("127.0.0.1", 8000))
        .attach(Router().middleware(session).route(home))
        .run()
    )


if __name__ == "__main__":
    main()
```

- `secret` (**bytes**): the key used to sign and verify the session cookie. Keep it long, random, and out of your source code (use an environment variable).
- `max_age`: session lifetime in seconds. Defaults to `604800` (1 week).

:::warning

Pick a strong, fixed secret and store it in an environment variable. If the secret changes, existing session cookies become invalid.

:::

## Reading and writing session data

The middleware injects a dictionary at `request.session`. Read and modify it like any dict:

```python
@get("/")
def home(request):
    visits = request.session.get("visits", 0) + 1
    request.session["visits"] = visits
    return {"visits": visits}
```

## How it works

1. On each request, the middleware reads the `session` cookie, verifies its HMAC-SHA256 signature and expiration, and stores the payload in `request.session`.
2. Your handler runs; changes to `request.session` are tracked.
3. If the session changed, the middleware signs the new data and adds a `Set-Cookie` header to the response:

```
session=<payload>.<signature>; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=604800
```

If the session was not modified, no cookie is written. Invalid or expired cookies are ignored and replaced by an empty session.

## Scoping sessions to routes

Like any middleware, the session middleware only applies to routes registered after it on the same router. Use separate routers when some routes should not have session support:

```python
(
    Oxapy(("127.0.0.1", 8000))
    .attach(
        Router()
        .route(get("/health", lambda _: "OK"))     # no session
        .middleware(session)
        .route(get("/profile", profile))            # session
    )
    .run()
)
```

See the [Middleware guide](./middleware) for more on scoping.

## Security notes

- The cookie is signed, not encrypted. Do not store sensitive data (passwords, credit card numbers) in the session.
- `Secure` is always set on the cookie, so sessions only work over HTTPS in production browsers.

## Next steps

- [Middleware](./middleware) — how middleware scoping works
- [JWT Authentication](./jwt-authentication) — an alternative for API token authentication
- [API Reference: Session](../api/session) — the `Session` signature and cookie format
