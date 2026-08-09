# JWT Authentication

The `jwt` submodule generates and verifies JSON Web Tokens with HMAC signing.

## Creating a Jwt instance

```python
from oxapy import jwt

jwt_handler = jwt.Jwt(secret="mysecret", algorithm="HS256")
```

- `secret` (str): the signing key. Use a long random value, kept in an environment variable.
- `algorithm` (str): defaults to `"HS256"`.

## Generating tokens

`generate_token(claims)` signs a claims dictionary and returns the token string.

```python
from oxapy import jwt, Router, post


jwt_handler = jwt.Jwt(secret="mysecret")
router = Router()


@post("/login")
def login(request):
    # Authenticate the user, then issue a token:
    claims = {
        "exp": 3600,            # lifetime in seconds FROM NOW (default 60)
        "sub": "user123",       # subject (optional)
        "iss": "myapp",         # issuer (optional)
        "aud": "webapp",        # audience (optional)
    }
    token = jwt_handler.generate_token(claims)
    return {"token": token}
```

:::note

The `exp` claim is interpreted as a number of **seconds from now**, not as an absolute Unix timestamp. When you omit it, tokens expire after 60 seconds. Any extra claims you include are preserved in the token.

:::

## Verifying tokens

`verify_token(token)` validates the signature and expiration, and returns the claims as a dictionary.

```python
from oxapy import exceptions, get, jwt

jwt_handler = jwt.Jwt(secret="mysecret")


@get("/protected")
def protected_route(request):
    token = request.headers.get("Authorization", "").replace("Bearer ", "")
    try:
        claims = jwt_handler.verify_token(token)
        return {"user_id": claims["sub"], "message": "Access granted"}
    except jwt.JwtDecodingError:
        raise exceptions.UnauthorizedError("Invalid or expired token")
```

Client call:

```bash
curl -H "Authorization: Bearer <token>" http://127.0.0.1:5555/protected
```

## Combining with middleware

Pair JWT verification with a middleware so protected routes share the logic:

```python
def require_auth(request, next, **kw):
    token = request.headers.get("Authorization", "").replace("Bearer ", "")
    try:
        request.user = jwt_handler.verify_token(token)
    except jwt.JwtError:
        return Status.UNAUTHORIZED
    return next(request, **kw)


router = (
    Router()
    .route(get("/health", lambda _: "OK"))   # public
    .middleware(require_auth)
    .route(get("/profile", profile))          # protected
)
```

## Errors

| Exception | Meaning |
| --- | --- |
| `JwtError` | Base class for all JWT errors |
| `JwtDecodingError` | Token is invalid, expired, or fails verification |
| `JwtInvalidAlgorithm` | Token algorithm does not match the configured one |
| `JwtInvalidClaim` | A claim is malformed |

`JwtEncodingError` is also exported for completeness, though token generation currently raises `JwtError` directly.

## Next steps

- [Middleware](./middleware) — protecting route groups
- [Sessions](./sessions) — the cookie-based alternative for browser apps
- [API Reference: JWT](../api/jwt) — full reference
