import os
import secrets
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
            return super().run(workers)

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


class Session:
    """
    Create a session middleware for signed, client-side cookie storage.

    This middleware extracts session data from a ``session`` cookie, verifies its
    HMAC-SHA256 signature, and injects the payload into ``request.session``.
    At the end of the request cycle, it compares the session state; if the
    dictionary was modified, it automatically signs the new data and inserts
    a ``Set-Cookie`` header into the response.

    Args:
        secret (bytes): The secret key used for HMAC signing and verification.
        max_age (int): Session expiration in seconds. Defaults to 1 week (604800s).
        samesite (str): SameSite cookie attribute. Defaults to ``"Lax"``.

    Returns:
        A middleware function to be registered via ``router.middleware()``.

    Example:
        ```python
        from oxapy import Oxapy, Session, Router, get


        @get("/")
        def home(request):
            request.session["visited"] = True
            return "Session updated"


        def main():
            session = Session(b"my-secret-key")
            (
                Oxapy(("127.0.0.1", 8000))
                .attach(Router().middleware(session).route(home))
                .run()
            )


        if __name__ == "__main__":
            main()
        ```
    """

    def __init__(self, secret: bytes, max_age: int = 3600 * 24 * 7, samesite="Lax"):
        self.secret = secret
        self.max_age = max_age
        self.samesite = samesite

    def __call__(self, request, next, **kwargs) -> Response:
        cookie = request.get_cookie("session")

        session_data = {}

        if cookie:
            verified = _verify_session(self.secret, cookie)
            if verified is not None:
                session_data = verified

        request.session = session_data
        initial_state = json.dumps(session_data)

        response = convert_to_response(next(request, **kwargs))

        current_state = json.dumps(request.session)
        if current_state != initial_state:
            signed_cookie = _sign_session(self.secret, self.max_age, request.session)

            response.set_cookie(
                name="session",
                value=signed_cookie,
                httponly=True,
                secure=True,
                samesite=self.samesite,
                max_age=self.max_age,
            )

        return response


def _generate_csrf_token(length: int = 32) -> str:
    return secrets.token_urlsafe(length)


def _sign_csrf_token(secret: bytes, token: str) -> str:
    signature = hmac.new(secret, token.encode(), hashlib.sha256).hexdigest()
    return f"{token}.{signature}"


def _verify_csrf_token(secret: bytes, signed: str) -> str | None:
    try:
        token, signature = signed.split(".", 1)
        expected = hmac.new(secret, token.encode(), hashlib.sha256).hexdigest()
        if not hmac.compare_digest(signature, expected):
            return None
        return token
    except Exception:
        return None


class CsrfProtect:
    """
    CSRF protection middleware using the Double Submit Cookie pattern.

    Validates a signed token on state-changing requests (POST, PUT, DELETE, PATCH)
    and sets a readable cookie on every response.

    The middleware stores the token on ``request.csrf_token``.  The ``render()``
    function automatically injects ``csrf_token`` into the template context, and
    the built-in ``csrf_input`` template function renders the hidden ``<input>``
    — no manual passing required.

    Args:
        secret (bytes): HMAC signing key for the token.
        cookie_name (str): Name of the cookie storing the signed token.
            Defaults to ``"csrf_token"``.
        header_name (str): Request header to check for the token (for AJAX).
            Defaults to ``"x-csrf-token"``.
        field_name (str): Form/JSON field name for the token.
            Defaults to ``"_csrf_token"``.
        cookie_max_age (int): Cookie lifetime in seconds. Defaults to 3600 (1 hour).
        safe_methods (tuple): HTTP methods that skip validation.
            Defaults to ``("GET", "HEAD", "OPTIONS", "TRACE")``.

    Returns:
        A middleware function to be registered via ``router.middleware()``.

    Example:
        ```python
        from oxapy import Oxapy, Router, CsrfProtect, get, post, render
        from oxapy import templating

        csrf = CsrfProtect(secret=b"my-secret-key")

        template = templating.Template()
        template.load("./templates/**/*.html")

        @get("/form")
        def form_view(request):
            return render(request, "form.html")

        @post("/submit")
        def submit(request):
            return {"status": "ok"}

        router = Router()
        router.middleware(csrf)
        router.routes([form_view, submit])

        Oxapy(("127.0.0.1", 8000)).template(template).attach(router).run()
        ```

    Templates:
        The ``csrf_input`` function is registered automatically. Use it with the
        injected ``csrf_token`` variable:

        ```html
        <form method="POST" action="/submit">
            {{ csrf_input(token=csrf_token) }}
            <input type="text" name="username">
            <button type="submit">Submit</button>
        </form>
        ```

    AJAX:
        Read the token from the cookie and send it as a header:

        ```javascript
        const token = document.cookie.match(/csrf_token=([^;]+)/)?.[1];
        fetch('/api/data', {
            method: 'POST',
            headers: { 'X-CSRF-Token': token },
            body: JSON.stringify({ key: 'value' })
        });
        ```
    """

    def __init__(
        self,
        secret: bytes,
        cookie_name: str = "csrf_token",
        header_name: str = "x-csrf-token",
        field_name: str = "_csrf_token",
        cookie_max_age: int = 3600,
        safe_methods: tuple[str, ...] = ("GET", "HEAD", "OPTIONS", "TRACE"),
    ):
        self.secret = secret
        self.cookie_name = cookie_name
        self.header_name = header_name
        self.field_name = field_name
        self.cookie_max_age = cookie_max_age
        self.safe_methods = safe_methods

    def __call__(self, request, next, **kwargs) -> Response:
        raw_cookie = request.get_cookie(self.cookie_name)
        token = None
        if raw_cookie:
            token = _verify_csrf_token(self.secret, raw_cookie)

        if token is None:
            token = _generate_csrf_token()

        request.csrf_token = token

        if request.method.upper() in self.safe_methods:
            response = convert_to_response(next(request, **kwargs))
        else:
            submitted = request.headers.get(self.header_name)
            if not submitted and self.field_name in request.form:
                submitted = request.form[self.field_name]
            if not submitted:
                try:
                    body = request.json()
                    if isinstance(body, dict):
                        submitted = body.get(self.field_name)
                except Exception:
                    pass

            if not submitted or not hmac.compare_digest(submitted, token):
                raise exceptions.ForbiddenError("CSRF token missing or invalid")

            response = convert_to_response(next(request, **kwargs))

        signed = _sign_csrf_token(self.secret, token)
        response.set_cookie(
            name=self.cookie_name,
            value=signed,
            max_age=self.cookie_max_age,
            httponly=False,
        )

        return response


def secure_join(base: str, *paths: str) -> str:
    base = os.path.realpath(base)
    target = os.path.realpath(os.path.join(base, *paths))

    if target != base and not target.startswith(base + os.sep):
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
    def handler(request: Request, path: str):
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
    "Oxapy",
    "Router",
    "Status",
    "Response",
    "Request",
    "Cors",
    "Session",
    "CsrfProtect",
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
    "convert_to_response",
    "templating",
    "serializer",
    "exceptions",
    "jwt",
)
