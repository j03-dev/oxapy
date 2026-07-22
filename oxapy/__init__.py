import os
import threading
import sys
import subprocess
import time
import base64
import typing
import mimetypes
import hmac
import orjson as json
import hashlib

from functools import partial

from watchdog.observers import Observer
from watchdog.events import PatternMatchingEventHandler

from .oxapy import *


class Oxapy(HttpServer):
    """
    An HTTP server extension that provides hot-reloading capabilities.

    When running with reload enabled, this class acts as a supervisor process
    that monitors file changes and automatically restarts the child worker process
    running the actual server.
    """

    def __new__(cls, addr: tuple[str, int]) -> "Oxapy":
        instance = super().__new__(cls, addr)
        instance.__patterns = ["*.py"]
        instance.__watch_dir = "."
        return instance

    def get_patterns(self):
        return self.__patterns

    def set_patterns(self, p: list[str]):
        """
        Sets the file patterns to monitor for changes.

        Args:
            p (list[str]): A list of glob patterns (e.g., ["*.py", "*.json"]).

        Returns:
            Oxapy: The current instance for method chaining.
        """
        self.__patterns = p
        return self

    def set_watch_dir(self, watch_dir: str):
        """
        Sets the base directory to watch for file modifications.

        Args:
            watch_dir (str): The directory path to watch (e.g., ".", "src/").

        Returns:
            Oxapy: The current instance for method chaining.
        """
        self.__watch_dir = watch_dir
        return self

    def run(self, reload: bool = False, workers: typing.Optional[int] = None):
        """
        Starts the server or the supervisor process.

        If `reload` is enabled and the current process is not flagged as a worker,
        it launches the supervisor to watch for file changes. Otherwise, it starts
        the actual HTTP server instance.

        Args:
            reload (bool): Whether to enable auto-reloading on file changes. Defaults to False.
            workers (int, optional): The number of worker processes to run. Defaults to None.
        """
        if reload and os.environ.get("OXAPY_WORKER") != "1":
            self._run_supervisor()
        else:
            super().run(workers)

    def _run_supervisor(self):
        """
        Manages the file watcher and the child worker process.

        Sets up a directory observer. When a watched file is modified, created,
        or deleted, it gracefully terminates the current worker process and
        spawns a fresh one.
        """
        env = os.environ.copy()
        env["OXAPY_WORKER"] = "1"

        reload_requested = threading.Event()
        changed_file_path = ""

        def on_file_changed(event):
            """Triggers a reload sequence when a watched file is modified."""
            nonlocal changed_file_path
            changed_file_path = event.src_path
            reload_requested.set()

        handler = PatternMatchingEventHandler(
            patterns=self.__patterns, ignore_directories=True
        )
        handler.on_modified = on_file_changed
        handler.on_created = on_file_changed
        handler.on_deleted = on_file_changed

        observer = Observer()
        observer.schedule(handler, self.__watch_dir, recursive=True)
        observer.start()

        def spawn_worker() -> subprocess.Popen:
            """Spawns the child server process with the worker environment flag."""
            return subprocess.Popen([sys.executable] + sys.argv, env=env)

        def terminate_worker(proc: subprocess.Popen):
            """Gracefully terminates a worker process, escalating to a kill if it hangs."""
            if proc and proc.poll() is None:
                proc.terminate()
                try:
                    proc.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    proc.kill()

        worker_process = spawn_worker()

        try:
            while True:
                if reload_requested.wait(timeout=0.2):
                    time.sleep(0.3)
                    reload_requested.clear()
                    terminate_worker(worker_process)
                    filename = os.path.basename(changed_file_path)
                    print(f"Reloading... ({filename} changed)")
                    worker_process = spawn_worker()
                elif worker_process.poll() is not None:
                    if worker_process.returncode != 0:
                        reload_requested.wait()
                        time.sleep(0.3)
                        reload_requested.clear()
                        worker_process = spawn_worker()
                    else:
                        break
        except KeyboardInterrupt:
            pass
        finally:
            observer.stop()
            observer.join()
            terminate_worker(worker_process)


def _b64_encode(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).decode().rstrip("=")


def _b64_decode(data: str) -> bytes:
    padding = "=" * (-len(data) % 4)
    return base64.urlsafe_b64decode(data + padding)


def _sign_session(secret: bytes, max_age: int, payload: dict[str, typing.Any]) -> str:
    body = {
        "data": payload,
        "exp": int(time.time()) + max_age,
    }

    json_data = json.dumps(body)

    payload_b64 = _b64_encode(json_data)

    signature = hmac.new(
        secret,
        payload_b64.encode(),
        hashlib.sha256,
    ).hexdigest()

    return f"{payload_b64}.{signature}"


def _verify_session(secret: bytes, cookie: str) -> dict[str, typing.Any] | None:
    try:
        payload_b64, signature = cookie.split(".", 1)

        expected_sig = hmac.new(
            secret,
            payload_b64.encode(),
            hashlib.sha256,
        ).hexdigest()

        if not hmac.compare_digest(signature, expected_sig):
            return None

        json_data = _b64_decode(payload_b64)
        body = json.loads(json_data)

        if body["exp"] < time.time():
            return None

        return body["data"]

    except Exception:
        return None


def _session_middleware(request, next, secret, max_age, **kwargs):
    cookie = request.get_cookie("session")

    session_data = {}

    if cookie:
        verified = _verify_session(secret, cookie)
        if verified is not None:
            session_data = verified

    request.session = session_data
    initial_state = json.dumps(session_data)

    response = convert_to_response(next(request, **kwargs))  # ty:ignore

    current_state = json.dumps(request.session)
    if current_state != initial_state:
        signed_cookie = _sign_session(secret, max_age, request.session)

        response.insert_header(
            "set-cookie",
            (
                f"session={signed_cookie}; "
                f"Path=/; "
                f"HttpOnly; "
                f"Secure; "
                f"SameSite=Lax; "
                f"Max-Age={max_age}"
            ),
        )

    return response


def Session(secret: bytes, max_age: int = 3600 * 24 * 7):
    r"""
    Create a session middleware for signed, client-side cookie storage.

    This middleware extracts session data from a "session" cookie, verifies its
    HMAC-SHA256 signature, and injects the payload into `request.session`.
    At the end of the request cycle, it compares the session state; if the
    dictionary was modified, it automatically signs the new data and inserts
    a "set-cookie" header into the response.

    Args:
        secret (bytes): The secret key used for HMAC signing and verification.
        max_age (int): Session expiration in seconds. Defaults to 1 week (604800s).

    Returns:
        functools.partial: A partially applied middleware function to be
                           attached via `.middleware()`.

    Example:
        ```python
        from oxapy import HttpServer, Session, Router, get


        @get("/")
        def home_view(request):
            # Modification triggers an automatic set-cookie in the response
            request.session["visited"] = True
            return "Session updated


        def main():
            session = Session(b"my-secret-key")
            (
                HttpServer(("0.0.0.0", 8000))
                .attach(
                    Router()
                    .middleware(session)
                    .routes([home_view])
                )
                .run()
            )

        if __name__ == "__main__":
            main()

    """
    return partial(_session_middleware, secret=secret, max_age=max_age)


def secure_join(base: str, *paths: str) -> str:
    base = os.path.realpath(base)
    target = os.path.realpath(os.path.join(base, *paths))

    if target != base and not target.startswith(base + os.sep):
        raise exceptions.ForbiddenError("Access denied")  # ty:ignore[unresolved-reference]

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

    @get(f"{path}/{{*path}}")  # ty:ignore[unresolved-reference]
    def handler(_request, path: str):
        file_path = secure_join(directory, path)
        return send_file(file_path)

    return handler


def send_file(path: str) -> Response:  # ty:ignore[unresolved-reference]
    r"""
    Create Response for sending file.

    Args:
        path (str): The full path to the file on the server's file system.
    Returns:
        Response: A Response with file content
    """
    if not os.path.exists(path):
        raise exceptions.NotFoundError("Requested file not found")  # type:ignore

    if not os.path.isfile(path):
        raise exceptions.ForbiddenError("Not a file")  # type:ignore

    with open(path, "rb") as f:
        content = f.read()
    content_type, _ = mimetypes.guess_type(path)
    return Response(content, content_type=content_type or "application/octet-stream")  # type:ignore


__all__ = (
    "HttpServer",
    "Oxapy",
    "Router",
    "Status",
    "Response",
    "Request",
    "Cors",
    "Session",
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
