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

The static route is then served at `/api/v1/static/...`.

## How it works

`static_file()` creates a `GET` catch-all route (`/static/{*path}`). For each request it:

1. Resolves the requested path inside the configured directory with `secure_join()`, which rejects path traversal attempts with `403 Forbidden`.
2. Reads the file with `send_file()`, raising `404 Not Found` when the file does not exist.
3. Guesses the `Content-Type` from the file extension.

```python
def send_file(path):
    if not os.path.exists(path):
        raise exceptions.NotFoundError("Requested file not found")

    if not os.path.isfile(path):
        raise exceptions.ForbiddenError("Not a file")

    content = open(path, "rb").read()
    content_type, _ = mimetypes.guess_type(path)
    return Response(content, content_type=content_type or "application/octet-stream")
```

## Serving individual files

To serve one specific file from a handler, use `send_file()`:

```python
from oxapy import get, send_file


@get("/report.pdf")
def report(request):
    return send_file("./files/report.pdf")
```

For very large files, prefer [FileStreaming](./file-streaming), which streams the file in chunks instead of loading it into memory.

## Next steps

- [File Streaming](./file-streaming) — chunked streaming of large files
- [API Reference: Response](../api/response) — building file responses by hand
