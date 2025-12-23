from .oxapy import *

import mimetypes


def static_file(path: str = "/static", directory: str = "./static"):
    r"""
    Create a route for serving static files.
    Args:
        directory (str): The directory containing static files.
        path (str): The URL path at which to serve the files.
    Returns:
        Route: A route configured to serve static files.
    Example:
    ```python
    from oxapy import Router, static_file
    router = Router()
    router.route(static_file("/static", "./static"))
    # This will serve files from ./static directory at /static URL path
    ```
    """

    @get(f"{path}/{{*path}}")
    def handler(_request, path: str):
        file_path = f"{directory}/{path}"
        try:
            return send_file(file_path)
        except FileNotFoundError:
            return Response("File not found", Status.NOT_FOUND)

    return handler


def send_file(path: str) -> Response:
    r"""
    Create Response for sending file.

    Args:
        path (str): The full path to the file on the server's file system.
    Returns:
        Response: A Response with file content
    """
    with open(path, "rb") as f:
        content = f.read()
    content_type, _ = mimetypes.guess_type(path)
    return Response(content, content_type=content_type or "application/octet-stream")


__all__ = (
    "HttpServer",
    "Router",
    "Status",
    "Response",
    "Request",
    "Cors",
    "Session",
    "SessionStore",
    "Redirect",
    "FileStreaming",
    "File",
    "get",
    "post",
    "delete",
    "patch",
    "put",
    "head",
    "options",
    "static_file",
    "render",
    "send_file",
    "catcher",
    "convert_to_response",
    "templating",
    "serializer",
    "exceptions",
    "jwt",
)
