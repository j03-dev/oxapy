# JWT

The `jwt` submodule generates and verifies JSON Web Tokens.

## Jwt

### Constructor

```python
Jwt(secret: str, algorithm: str = "HS256")
```

- `secret` — signing key
- `algorithm` — signing algorithm, default `"HS256"`

```python
from oxapy import jwt

jwt_handler = jwt.Jwt(secret="mysecret", algorithm="HS256")
```

### generate_token

```python
generate_token(claims: dict) -> str
```

Signs `claims` and returns the token string. The `exp` claim is a lifetime in **seconds from now** (default `60` when omitted):

```python
token = jwt_handler.generate_token({"exp": 3600, "sub": "user123", "role": "admin"})
```

Other standard claims (`sub`, `iss`, `aud`, `nbf`) and any extra claims are preserved in the token.

### verify_token

```python
verify_token(token: str) -> dict
```

Validates the signature and expiration and returns the claims dictionary:

```python
claims = jwt_handler.verify_token(token)
print(claims["sub"])
```

Raises `JwtDecodingError` when the token is invalid or expired.

## Exceptions

| Exception | Description |
| --- | --- |
| `JwtError` | Base class for all JWT errors |
| `JwtDecodingError` | Token could not be decoded or verified (expired, malformed) |
| `JwtEncodingError` | Encoding errors (exported; generation currently raises `JwtError`) |
| `JwtInvalidAlgorithm` | Algorithm mismatch |
| `JwtInvalidClaim` | Malformed claim |

## Example: protecting a route

```python
from oxapy import exceptions, get, jwt

jwt_handler = jwt.Jwt(secret="mysecret")


@get("/protected")
def protected(request):
    token = request.headers.get("Authorization", "").replace("Bearer ", "")
    try:
        claims = jwt_handler.verify_token(token)
        return {"user_id": claims["sub"], "message": "Access granted"}
    except jwt.JwtDecodingError:
        raise exceptions.UnauthorizedError("Invalid or expired token")
```

## Related

- [JWT Authentication guide](../guides/jwt-authentication) — full walkthrough
- [Exceptions](./exceptions) — HTTP exception classes
