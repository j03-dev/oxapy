# Exceptions

The `exceptions` module provides typed HTTP errors. Raise them from handlers and OxAPY converts them into JSON error responses.

## Hierarchy

```text
Exception
├── ClientError          # base for all 4xx errors
│   ├── BadRequestError      → 400
│   ├── UnauthorizedError    → 401
│   ├── ForbiddenError       → 403
│   ├── NotFoundError        → 404
│   └── ConflictError        → 409
└── InternalError        → 500
```

## Usage

```python
from oxapy import Router, get, exceptions


@get("/users/{user_id}")
def get_user(request, user_id: int):
    user = find_user(user_id)
    if user is None:
        raise exceptions.NotFoundError("User not found")
    return user
```

Response: `404` with body:

```json
{"detail": "User not found"}
```

## Status mapping

| Exception | Status |
| --- | --- |
| `UnauthorizedError` | 401 |
| `ForbiddenError` | 403 |
| `NotFoundError` | 404 |
| `ConflictError` | 409 |
| Other `ClientError` subclasses | 400 |
| Anything else (including `InternalError`) | 500 |

## Related

- [Error Handling guide](../guides/error-handling) — full walkthrough
- [Status](./status) — returning status codes directly
- [JWT](./jwt) — `JwtError` and its subclasses
