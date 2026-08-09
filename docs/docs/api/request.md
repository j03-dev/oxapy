# Request & File

The `Request` object is passed to every handler as the first argument. `File` represents an uploaded file from a multipart form.

## Request

### Properties

| Property | Type | Description |
| --- | --- | --- |
| `method` | `str` | HTTP method (GET, POST, ...) |
| `uri` | `str` | Full URI including the query string |
| `headers` | `dict[str, str]` | HTTP headers |
| `data` | `str \| None` | Raw body content, when present |
| `form` | `dict[str, str]` | Form fields for `application/x-www-form-urlencoded` bodies |
| `files` | `dict[str, File]` | Uploaded files keyed by field name |
| `app_data` | `Any` | Application-wide data set with `HttpServer.app_data()` |
| `query` | `dict[str, str]` | Parsed query string parameters |

```python
@get("/debug")
def debug(request):
    return {
        "method": request.method,
        "uri": request.uri,
        "user_agent": request.headers.get("user-agent"),
        "query": request.query,
    }
```

### Methods

#### json

```python
json() -> dict
```

Parses the request body as JSON. Raises an exception when the body is missing or not valid JSON.

```python
@post("/api/data")
def create_data(request):
    data = request.json()
    return {"received": data}
```

#### get_cookie

```python
get_cookie(name: str) -> str | None
```

Returns the value of the named cookie, or `None` if it is not present.

```python
theme = request.get_cookie("theme") or "light"
```

### Dynamic attributes

Requests support arbitrary attribute assignment, which is how middleware passes data to handlers:

```python
def auth_middleware(request, next, **kw):
    request.user_name = "John Doe"
    return next(request, **kw)
```

## File

### Properties

| Property | Type | Description |
| --- | --- | --- |
| `name` | `str` | Original file name |
| `content_type` | `str` | Uploaded file MIME type |
| `content` | `bytes` | Full file content |

### save

```python
save(path: str) -> None
```

Writes the uploaded file to `path`.

```python
@post("/upload")
def upload(request):
    image = request.files["profile_image"]
    image.save(f"uploads/{image.name}")
    return {"filename": image.name}
```

## Example: full request handling

```python
from oxapy import post


@post("/submit")
def submit(request):
    result = {
        "json": request.json(),
        "form": dict(request.form),
        "files": {
            name: {"name": f.name, "size": len(f.content)}
            for name, f in request.files.items()
        },
    }
    return result
```

## Related

- [Requests guide](../guides/requests) — examples for every property
- [Response](./response) — what handlers return
