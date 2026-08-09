# Session

`Session` creates a middleware for signed, client-side cookie sessions. Session data lives in a `session` cookie signed with HMAC-SHA256, so no server-side storage is needed.

## Signature

```python
Session(secret: bytes, max_age: int = 604800) -> middleware
```

- `secret` (**bytes**) — the HMAC signing key
- `max_age` — session lifetime in seconds, default `604800` (1 week)

The return value is a partially-applied middleware, registered with `router.middleware()`.

## Example

```python
from oxapy import Oxapy, Router, Session, get


@get("/")
def home(request):
    request.session["visited"] = True
    return {"session": dict(request.session)}


def main():
    session = Session(b"my-secret-key")
    (
        Oxapy(("0.0.0.0", 8000))
        .attach(Router().middleware(session).routes([home]))
        .run()
    )


if __name__ == "__main__":
    main()
```

## Request API

The middleware injects a dictionary at `request.session`:

- Read session values: `request.session.get("key", default)`
- Write session values: `request.session["key"] = value`
- Any modification triggers a signed `Set-Cookie` header on the response

## Cookie format

```
session=<urlsafe-base64(payload)>.<hex-hmac-sha256-signature>; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=<max_age>
```

The payload is `{"data": <session dict>, "exp": <unix timestamp>}`. Invalid signatures or expired payloads are discarded and replaced with an empty session.

## Notes

- The cookie is signed, not encrypted — do not store secrets in it.
- If the session dict is not modified during a request, no `Set-Cookie` header is written.
- The `Secure` flag means browsers only send the cookie over HTTPS.

## Related

- [Sessions guide](../guides/sessions) — full walkthrough
- [Middleware guide](../guides/middleware) — how middleware is scoped
- [JWT](./jwt) — token-based alternative for APIs
