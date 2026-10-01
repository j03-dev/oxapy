# Static Files

Serve static assets with `static_file()`, which creates a route that maps a URL path to a directory on disk.

## Serving a directory

```python
from oxapy import Oxapy, Router, static_file


def main():
    (
        Oxapy(("127.0.0.1", 5555))
        .attach(Router().route(static_file("/static", "./static")))
        .run()
    )


if __name__ == "__main__":
    main()
```

Files from `./static` are now served under `/static`. For example, `./static/index.html` is available at `http://127.0.0.1:5555/static/index.html`.

Both parameters have defaults: `static_file(path="/static", directory="./static")`.

## Inside a router with a base path

Because `static_file()` returns a `Route`, it can be registered alongside normal routes — including on a router with a base path:

```python
router = Router("/api/v1").routes([ping, hello, static_file("/static", str(static_dir))])
```

The static route is then served at `/api/v1/static/...`. Router paths are normalized when they are registered, so redundant slashes in the combined prefix collapse rather than producing a route that never matches.

## How it works

`static_file()` creates a `GET` catch-all route (`/static/{*path}`). For each request it:

1. Resolves the requested path inside the configured directory with `secure_join()`, which rejects path traversal attempts with `403 Forbidden`.
2. Reads the file with `send_file()`, raising `404 Not Found` when the file does not exist.
3. Guesses the `Content-Type` from the file extension.

`secure_join()` resolves both the base directory and the joined target through `os.path.realpath` before comparing them, so the check also rejects a symlink inside the directory that points outside it:

```python
def secure_join(base, *paths):
    base = os.path.realpath(base)
    target = os.path.realpath(os.path.join(base, *paths))

    if target != base and not target.startswith(base + os.sep):
        raise exceptions.ForbiddenError("Access denied")

    return target
```

`send_file()` then reads the file and guesses its content type:

```python
def send_file(path):
    if not os.path.exists(path):
        raise exceptions.NotFoundError("Requested file not found")

    if not os.path.isfile(path):
        raise exceptions.ForbiddenError("Not a file")

    with open(path, "rb") as f:
        content = f.read()
    content_type, _ = mimetypes.guess_type(path)
    return Response(content, content_type=content_type or "application/octet-stream")
```

Note that `send_file()` reads the whole file into memory, and it applies **no** traversal check of its own — only the `secure_join()` call inside `static_file()` protects the route. When you call `send_file()` from your own handler, resolve user input through `secure_join()` first. See the [Static Files API reference](../api/static-files).

## Serving individual files

To serve one specific file from a handler, use `send_file()`:

```python
from oxapy import get, send_file


@get("/report.pdf")
def report(request):
    return send_file("./files/report.pdf")
```

For very large files, prefer [File Streaming](./file-streaming), which streams the file in chunks instead of loading it into memory. `FileStreaming` performs no path validation either — validate the path yourself before handing it over.

## Next steps

- [File Streaming](./file-streaming) — chunked streaming of large files
- [Static Files API](../api/static-files) — `static_file`, `send_file`, and `secure_join` reference
- [API Reference: Response](../api/response) — building file responses by hand
