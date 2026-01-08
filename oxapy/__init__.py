from .oxapy import *

import os
import mimetypes


def secure_join(base: str, user_path: str) -> str:
    base = os.path.abspath(base)
    target = os.path.normpath(os.path.join(base, user_path))

    if not target.startswith(base + os.sep):
        raise exceptions.ForbiddenError("Access denied")

    return target


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
        file_path = secure_join(directory, path)
        return send_file(file_path)

    return handler


def send_file(path: str) -> Response:
    r"""
    Create Response for sending file.

    Args:
        path (str): The full path to the file on the server's file system.
    Returns:
        Response: A Response with file content
    """
    if not os.path.exists(path):
        raise exceptions.NotFoundError("Requested file not found")

    if not os.path.isfile(path):
        raise exceptions.ForbiddenError("Not a file")

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
