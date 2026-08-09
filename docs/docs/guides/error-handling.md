# Error Handling

Raise typed exceptions from your handlers and OxAPY converts them into JSON error responses automatically.

## The exceptions module

```python
from oxapy import exceptions
```

| Exception | HTTP status | Typical use |
| --- | --- | --- |
| `BadRequestError` | 400 | Malformed input |
| `UnauthorizedError` | 401 | Missing or failed authentication |
| `ForbiddenError` | 403 | Authenticated but not allowed |
| `NotFoundError` | 404 | Resource does not exist |
| `ConflictError` | 409 | Conflict, e.g. duplicate resource |
| `InternalError` | 500 | Unexpected server failure |

`ClientError` is the base class of all 4xx exceptions.

## Raising errors

```python
from oxapy import Router, get, exceptions


@get("/users/{user_id}")
def get_user(request, user_id: int):
    user = find_user(user_id)
    if user is None:
        raise exceptions.NotFoundError("User not found")
    return user
```

The response looks like:

```json
{"detail": "User not found"}
```

with status `404`.

## Mapping details

Raised exceptions are mapped to status codes as follows:

- `UnauthorizedError` → 401
- `ForbiddenError` → 403
- `NotFoundError` → 404
- `ConflictError` → 409
- other `ClientError` subclasses → 400
- anything else (including unexpected Python exceptions) → 500

The exception message becomes the `detail` field of a JSON body.

:::note

When the `DEBUG` environment variable is set, unexpected exceptions are also printed to the server log, which helps during development.

:::

## Returning status codes directly

For simple cases you can return a `Status` value instead of raising:

```python
from oxapy import Status, get


@get("/admin")
def admin(request):
    if not is_admin(request):
        return Status.FORBIDDEN
    return {"secret": 42}
```

The response has an empty JSON body. See [Responses](./responses) for details.

## Wrapping responses globally

`server.wrap(callable)` installs a global wrapper that runs after every handler with `(request, response)` and can modify or replace the response:

```python
from oxapy import Oxapy, Response


def global_wrapper(request, response):
    if response.status == Status.NOT_FOUND:
        return Response("<h1>Page Not Found</h1>", content_type="text/html")
    return response


server = Oxapy(("127.0.0.1", 8000))
server.wrap(global_wrapper)
```

The wrapper's return value is converted with the same rules as a handler's return value.

## Next steps

- [Responses](./responses) — building responses by hand
- [API Reference: Exceptions](../api/exceptions) — the full exception hierarchy
