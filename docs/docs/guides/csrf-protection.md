# CSRF Protection

Cross-Site Request Forgery (CSRF) attacks trick a logged-in user's browser into sending unwanted requests to your server. OxAPY provides a `CsrfProtect` middleware that uses the **Double Submit Cookie** pattern to guard state-changing requests.

## Enabling CSRF protection

Create a `CsrfProtect` middleware with a secret key and register it on a router:

```python
from oxapy import Oxapy, Router, CsrfProtect, get, post

csrf = CsrfProtect(secret=b"my-secret-key")

@post("/transfer")
def transfer(request):
    return {"status": "ok"}

def main():
    (
        Oxapy(("127.0.0.1", 8000))
        .attach(Router().middleware(csrf).route(transfer))
        .run()
    )
```

- `secret` (**bytes**): HMAC signing key for the token. Keep it long, random, and out of source code.
- Safe methods (`GET`, `HEAD`, `OPTIONS`, `TRACE`) skip validation automatically.

:::warning

Pick a strong, fixed secret and store it in an environment variable. If the secret changes, existing CSRF tokens become invalid.

:::

## How it works

1. On every request the middleware generates or verifies a signed CSRF token.
2. The token is stored on `request.csrf_token` and set as a readable cookie (`csrf_token`).
3. On state-changing methods (`POST`, `PUT`, `DELETE`, `PATCH`) the middleware checks the token from one of three sources:
   - **Header**: `X-CSRF-Token` (for AJAX/fetch requests)
   - **Form field**: `_csrf_token` (for HTML forms)
   - **JSON body**: `_csrf_token` key (for JSON APIs)
4. If the token is missing or invalid, the middleware raises `ForbiddenError` (403).

## Using with templates

The `render()` function automatically injects `csrf_token` into the template context. The built-in `csrf_input` template function renders the hidden `<input>`:

```html
<form method="POST" action="/transfer">
  {{ csrf_input(token=csrf_token) }}
  <input type="text" name="amount" />
  <button type="submit">Transfer</button>
</form>
```

This outputs:

```html
<input type="hidden" name="_csrf_token" value="..." />
```

No manual passing of the token from handler to template is needed — `render()` handles it.

## Using with AJAX / fetch

Read the token from the cookie and send it as a request header:

```javascript
const token = document.cookie.match(/csrf_token=([^;]+)/)?.[1];

fetch("/api/data", {
  method: "POST",
  headers: { "X-CSRF-Token": token },
  body: JSON.stringify({ key: "value" }),
});
```

## Scoping CSRF to routes

Like any middleware, `CsrfProtect` only applies to routes registered after it on the same router. Use separate routers when some routes should not have CSRF protection:

```python
(
    Oxapy(("127.0.0.1", 8000))
    .attach(
        Router()
        .route(get("/health", lambda _: "OK"))        # no CSRF
        .middleware(csrf)
        .routes([form_view, transfer])                 # CSRF protected
    )
    .run()
)
```

See the [Middleware guide](./middleware) for more on scoping.

## Configuration

```python
CsrfProtect(
    secret=b"my-secret-key",
    cookie_name="csrf_token",       # cookie name (default: "csrf_token")
    header_name="x-csrf-token",     # header name for AJAX (default: "x-csrf-token")
    field_name="_csrf_token",       # form/JSON field name (default: "_csrf_token")
    cookie_max_age=3600,            # cookie lifetime in seconds (default: 1 hour)
    safe_methods=("GET", "HEAD", "OPTIONS", "TRACE"),  # methods that skip validation
)
```

## Combining with sessions

CSRF protection works alongside the `Session` middleware. Register both on the same router:

```python
session = Session(b"session-secret")
csrf = CsrfProtect(b"csrf-secret")

(
    Oxapy(("127.0.0.1", 8000))
    .attach(
        Router()
        .middleware(session)
        .middleware(csrf)
        .routes([form_view, submit])
    )
    .run()
)
```

## Next steps

- [Sessions](./sessions) — signed cookie sessions
- [Middleware](./middleware) — how middleware scoping works
- [Templates](./templates) — rendering HTML with Tera
- [Error Handling](./error-handling) — handling 403 and other errors
