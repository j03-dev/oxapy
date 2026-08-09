# File Streaming

`FileStreaming` serves large files in chunks instead of loading the entire file into memory. Use it for videos, PDFs, archives, or any download where memory matters.

## Streaming a file

```python
from oxapy import Router, FileStreaming, get


@get("/downloads/{filename}")
def download(request, filename):
    return FileStreaming(f"./downloads/{filename}")
```

Signature: `FileStreaming(path, buf_size=8192, status=Status.OK, content_type="application/octet-stream")`.

The file is read in chunks of `buf_size` bytes (default 8 KB). The response includes `Cache-Control: no-cache`.

## Streaming with a catch-all route

Pair `FileStreaming` with a `{*path}` route to serve a whole directory tree:

```python
@get("/videos/{*path}")
def serve_video(request, path):
    return FileStreaming(
        f"./media/videos/{path}",
        buf_size=16384,          # 16 KB chunks
        content_type="video/mp4",
    )
```

A request to `/videos/clips/intro.mp4` streams `./media/videos/clips/intro.mp4`.

## Tuning the buffer size

Larger buffers can improve throughput for big files at the cost of more memory per request:

```python
@get("/files/{filename}")
def serve_big_file(request, filename):
    return FileStreaming(
        f"./files/{filename}",
        buf_size=65536,  # 64 KB chunks
        content_type="application/pdf",
    )
```

## Errors

`FileStreaming` raises `OSError` (or `PermissionError`) if the file cannot be opened or read, and `ValueError` if the path is invalid. If the file may not exist, check first and return a 404 yourself:

```python
import os

from oxapy import Router, FileStreaming, Status, get


@get("/media/{name}")
def media(request, name):
    path = f"./media/{name}"
    if not os.path.exists(path):
        return Status.NOT_FOUND
    return FileStreaming(path)
```

## Compared with send_file

- `send_file()` reads the whole file into memory and is a good fit for small assets.
- `FileStreaming()` streams in chunks and is the right choice for large files.

Static assets served through [static_file()](./static-files) use `send_file()` under the hood.

## Next steps

- [Static Files](./static-files) — serving directories with `static_file()`
- [Responses](./responses) — other response types
- [API Reference: Response](../api/response) — `FileStreaming` details
