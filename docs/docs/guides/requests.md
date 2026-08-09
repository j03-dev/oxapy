# Requests

Every handler receives a `Request` object as its first argument. It exposes the method, URI, headers, body, and parsed data.

## Method and URI

```python
from oxapy import get


@get("/debug")
def debug(request):
    return {
        "method": request.method,  # "GET"
        "uri": request.uri,        # full URI including query string
    }
```

## Headers

Headers are available as a plain dictionary:

```python
@get("/hello")
def hello(request):
    user_agent = request.headers.get("user-agent")
    return f"Hello from {user_agent}"
```

## Query parameters

`request.query` parses the query string of the URI into a dictionary. Unknown keys fall back to a default with `dict.get`.

```python
from oxapy import get


@get("/search")
def search(request):
    q = request.query.get("q", "")
    limit = int(request.query.get("limit", "20"))
    return {"q": q, "limit": limit}
```

For `/search?q=rust&limit=10` this returns `{"q": "rust", "limit": 10}`.

## JSON bodies

Use `request.json()` to parse a JSON request body. It is the standard way to read `POST`/`PUT` payloads.

```python
from oxapy import post


@post("/api/data")
def create_data(request):
    data = request.json()
    return {"status": "success", "received": data}
```

`request.data` contains the raw body as a string, when present.

## Forms

For `application/x-www-form-urlencoded` bodies, `request.form` gives you a dictionary of the submitted fields.

```python
from oxapy import post


@post("/login")
def login(request):
    form = request.form
    return {"username": form["username"]}
```

## File uploads

Multipart form data is exposed through `request.files`, a dictionary mapping field names to `File` objects.

```python
from oxapy import post


@post("/upload")
def upload(request):
    files_info = {}
    for name, file in request.files.items():
        files_info[name] = {
            "filename": file.name,
            "content_type": file.content_type,
            "size": len(file.content),
        }
    return {"files": files_info, "form": dict(request.form)}
```

### Saving an uploaded file

```python
@post("/upload")
def upload(request):
    if "profile_image" in request.files:
        image = request.files["profile_image"]
        image.save(f"uploads/{image.name}")
        return {"status": "success", "filename": image.name}
    return {"status": "error", "message": "No file uploaded"}
```

## Cookies

Read cookies with `request.get_cookie(name)`, which returns `None` when the cookie is absent.

```python
@get("/")
def index(request):
    theme = request.get_cookie("theme") or "light"
    return {"theme": theme}
```

## Application data

`request.app_data` returns the object you set with `HttpServer.app_data()`. It is shared across all requests, which makes it the right place for counters, pools, or other shared resources. See the [Application State guide](./app-state).

```python
@get("/count")
def count(request):
    state = request.app_data
    state.counter += 1
    return {"count": state.counter}
```

## Dynamic attributes

Middleware and handlers can attach arbitrary attributes to a request. For example, an authentication middleware may store the current user:

```python
def auth_middleware(request, next, **kw):
    request.user_name = "John Doe"
    return next(request, **kw)


@get("/profile")
def profile(request):
    return {"user": request.user_name}
```

See the [Middleware guide](./middleware) for details.

## Next steps

- [Responses](./responses) — what to return from a handler
- [Middleware](./middleware) — request lifecycle hooks
- [API Reference: Request](../api/request) — every property and method
