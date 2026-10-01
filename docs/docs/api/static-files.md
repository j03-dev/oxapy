# Static Files API

Three helpers back OxAPY's file-serving features: `static_file()` builds a route, `send_file()` turns a path into a `Response`, and `secure_join()` is the path traversal guard both of them rely on.

| Function | Purpose |
| --- | --- |
| [`static_file`](#static_file) | Build a `Route` that serves a directory |
| [`send_file`](#send_file) | Return a `Response` containing one file |
| [`secure_join`](#secure_join) | Resolve a path inside a base directory, or raise |

## static_file

```python
static_file(path: str = "/static", directory: str = "./static") -> Route
```

Creates a `GET` catch-all route that serves `directory` under the URL prefix `path`.

| Parameter | Default | Description |
| --- | --- | --- |
| `path` | `"/static"` | URL prefix the files are served under |
| `directory` | `"./static"` | Directory on disk to read from |

```python
from oxapy import Oxapy, Router, static_file

(
    Oxapy(("127.0.0.1", 5555))
    .attach(Router().route(static_file("/static", "./static")))
    .run()
)
```

The registered route pattern is `f"{path}/{{*path}}"`, so the default is `/static/{*path}` — a catch-all matching one or more segments. `./static/index.html` is then served at `/static/index.html`.

Returns a `Route`, so it can be registered with `.route()` or included in a list passed to `.routes()`. Because it is an ordinary route, a router's `base_path` is prepended to it: on `Router("/api/v1")` the files land under `/api/v1/static/...`.

See the [Static Files guide](../guides/static-files) for worked examples.

## send_file

```python
send_file(path: str) -> Response
```

Returns a `Response` containing the file at `path`.

| Behavior | Result |
| --- | --- |
| Path does not exist | raises `NotFoundError` → **404 Not Found** |
| Path is a directory | raises `ForbiddenError` → **403 Forbidden** |
| Content type | guessed from the extension via `mimetypes`, falling back to `application/octet-stream` |
| Status | `200 OK` (the default) |

```python
from oxapy import get, send_file


@get("/report.pdf")
def report(request):
    return send_file("./files/report.pdf")
```

:::note The whole file is read into memory

`send_file()` loads the entire file into a single buffer before responding. For large files prefer [`FileStreaming`](../guides/file-streaming), which reads in chunks. `send_file()` also performs **no** traversal protection — it opens whatever path you give it.

:::

## secure_join

```python
secure_join(base: str, *paths: str) -> str
```

Resolves `paths` inside `base` and returns the absolute path, raising `ForbiddenError` (**403**) if the result escapes `base`.

Both `base` and the joined target are passed through `os.path.realpath` first, so the check is **symlink-aware** — a symlink inside `base` that points outside it is rejected rather than followed.

```python
from oxapy import secure_join

secure_join("./static", "css/app.css")          # -> '/srv/app/static/css/app.css'
secure_join("./static", "../../etc/passwd")     # raises ForbiddenError('Access denied')
secure_join("./static", "link -> /etc/passwd")  # raises ForbiddenError('Access denied')
```

The rule is that the resolved target must either equal `base` or start with `base + os.sep`.

`secure_join` is importable from `oxapy` but is not listed in the package's `__all__`; treat it as a public helper for your own file-serving handlers rather than a documented export.

### Using it in your own handlers

Any handler that builds a filesystem path from user input should route it through `secure_join`:

```python
from oxapy import Router, get, secure_join, send_file

UPLOADS = "./uploads"


@get("/download/{*path}")
def download(request, path):
    # Rejects ../ traversal and escaping symlinks with 403
    return send_file(secure_join(UPLOADS, path))
```

:::warning `secure_join` does not sanitize filenames

It answers "is this path inside the base directory?", not "is this name safe to create?". When **writing** files — for example handling an upload — also strip any directory component from the client-supplied name, otherwise the result is still attacker-controlled:

```python
import os

safe_name = os.path.basename(image.name)
image.save(os.path.join(UPLOADS, safe_name))
```

See the [Requests guide](../guides/requests#saving-an-uploaded-file) for the full pattern.

:::

## Errors

Both `NotFoundError` and `ForbiddenError` come from [`oxapy.exceptions`](../api/exceptions) and are mapped to HTTP responses automatically, so you do not need to catch them:

| Exception | HTTP status |
| --- | --- |
| `NotFoundError` | `404 Not Found` |
| `ForbiddenError` | `403 Forbidden` |

## Related

- [Static Files guide](../guides/static-files) — serving directories and single files
- [File Streaming](../guides/file-streaming) — chunked serving for large files
- [Exceptions](./exceptions) — the full exception hierarchy