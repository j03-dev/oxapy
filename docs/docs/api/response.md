# Response & Redirect

Handlers return values that OxAPY converts into `Response` objects. The `Response` class gives you full control over body, status, and headers.

## Response

### Constructor

```python
Response(body, status: Status = Status.OK, content_type: str = "application/json")
```

- `body` — a string, bytes, or JSON-serializable object
- `status` — a `Status` member, defaults to `OK`
- `content_type` — the `Content-Type` header, defaults to `"application/json"`

```python
from oxapy import Response, Status

Response({"message": "Success"})                     # JSON
Response("Hello", content_type="text/plain")         # raw text
Response("Not found", status=Status.NOT_FOUND)       # 404, JSON-encoded body
Response(b"\x89PNG...", content_type="image/png")    # raw bytes
```

:::note

With the default `application/json` content type the body is serialized with `orjson`, so a string body is JSON-encoded (including quotes). Use `content_type="text/plain"` for raw text and a non-JSON content type for raw bytes.

:::

### Properties

| Property | Type | Description |
| --- | --- | --- |
| `status` | `Status` | The response status; settable |
| `body` | `str` | The response body as a UTF-8 string |
| `headers` | `list[tuple[str, str]]` | Headers as key-value tuples |

### Methods

#### insert_header

```python
insert_header(key: str, value: str) -> None
```

Adds or replaces a header.

```python
response.insert_header("Cache-Control", "no-cache")
```

#### append_header

```python
append_header(key: str, value: str) -> None
```

Appends a value to a repeatable header such as `Set-Cookie`.

```python
response.append_header("Set-Cookie", "sessionid=abc123")
response.append_header("Set-Cookie", "theme=dark")
```

## Redirect

### Constructor

```python
Redirect(location: str)
```

A `Response` subclass that issues a `301 Moved Permanently` redirect.

```python
from oxapy import Redirect, get


@get("/old")
def old(request):
    return Redirect("/new")
```

## Handler return values

The server converts handler results with `convert_to_response`:

| Return value | Result |
| --- | --- |
| `Response` | Used as-is |
| `str` | `text/plain` response |
| `dict` / JSON-serializable object | JSON response |
| `Status` | JSON response with an empty body and that status |
| `(str, Status)` | `text/plain` body with the given status |
| `(obj, Status)` | JSON body with the given status |

Anything else raises a `ValueError`.

## FileStreaming

### Constructor

```python
FileStreaming(path: str, buf_size: int = 8192, status: Status = Status.OK, content_type: str = "application/octet-stream")
```

Streams a file in chunks for large files without loading the entire file into memory.

| Parameter | Default | Description |
| --- | --- | --- |
| `path` | — | Path to the file |
| `buf_size` | `8192` | Read buffer size in bytes |
| `status` | `Status.OK` | Response status code |
| `content_type` | `"application/octet-stream"` | The `Content-Type` header |

```python
from oxapy import FileStreaming

@get("/download")
def download(request):
    return FileStreaming("report.pdf", content_type="application/pdf")
```

See the [File Streaming guide](../guides/file-streaming) for details.

## Related

- [Responses guide](../guides/responses) — examples and return types
- [Status](./status) — the status code enum
