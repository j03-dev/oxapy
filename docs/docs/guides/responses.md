# Responses

Handlers can return several kinds of values. OxAPY converts them into HTTP responses automatically.

## Return types

| Return value | Result |
| --- | --- |
| `str` | Plain text (`text/plain`) |
| `dict` or JSON-serializable object | JSON body (`application/json`) |
| `Response` | Used as-is |
| `Status` | JSON response with an empty body and that status code |
| `(str, Status)` | Plain text with the given status |
| `(obj, Status)` | JSON body with the given status |

```python
from oxapy import Router, get, post, Status


@get("/text")
def text(request):
    return "Hello, World!"  # text/plain


@get("/json")
def json_data(request):
    return {"message": "Hello", "count": 3}  # application/json


@get("/gone")
def gone(request):
    return Status.GONE  # 410, empty JSON body


@post("/create")
def create(request):
    return ("Created", Status.CREATED)  # text/plain, 201
```

:::warning

A handler that returns an unsupported type raises a `ValueError`. If you need a custom body, build an explicit `Response`.

:::

## The Response class

`Response` gives you full control over the body, status, and headers.

```python
from oxapy import Response, Status

# JSON response
Response({"message": "Success"})

# Plain text
Response("Hello, World!", content_type="text/plain")

# Custom status
Response("Not authorized", status=Status.UNAUTHORIZED)

# HTML
Response("<h1>Not Found</h1>", Status.NOT_FOUND, "text/html")
```

Signature: `Response(body, status=Status.OK, content_type="application/json")`.

:::note

The default `content_type` is `application/json`, so a string body is JSON-encoded (quotes included) unless you pass `content_type="text/plain"`. Bytes bodies with a non-JSON content type are sent raw, which is how `send_file` serves files.

:::

### Headers

Use `insert_header()` to set a header and `append_header()` to add another value to a repeatable header such as `Set-Cookie`.

```python
from oxapy import Response


def handler(request):
    response = Response({"ok": True})
    response.insert_header("Cache-Control", "no-cache")
    response.append_header("Set-Cookie", "sessionid=abc123")
    response.append_header("Set-Cookie", "theme=dark")
    return response
```

`response.headers` returns the headers as a list of `(name, value)` tuples, and `response.body` returns the body as a string.

## Redirects

`Redirect(location)` produces a `301 Moved Permanently` response pointing at another URL.

```python
from oxapy import Redirect, get


@get("/old")
def old_page(request):
    return Redirect("/new")


@get("/external")
def external(request):
    return Redirect("https://example.com")
```

## Status responses

Return a `Status` enum member directly to answer with that code and an empty JSON body. The enum supports comparison operators, so you can check ranges:

```python
from oxapy import Status


def handler(request):
    status = Status.NOT_FOUND
    if status == Status.NOT_FOUND:
        ...  # handle it
    return status
```

See the [Status reference](../api/status) for the complete list of codes. When you raise an `exceptions` subclass instead, the server builds the response for you — see [Error Handling](./error-handling).

## Next steps

- [Error Handling](./error-handling) — raising typed exceptions that map to HTTP errors
- [File Streaming](./file-streaming) — streaming large files with `FileStreaming`
- [API Reference: Response](../api/response) — every property and method
