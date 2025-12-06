from oxapy import HttpServer, Router, get, post, static_file, Status, Response
import threading
import time
import pytest
from pathlib import Path


# App State
class AppState:
    def __init__(self):
        self.counter = 0


# Middleware
def auth_middleware(request, next, **kw):
    if "authorization" not in request.headers:
        return Status.UNAUTHORIZED
    request.user_name = "John Does"
    return next(request, **kw)


# Handlers
@get("/hello/{name}")
def hello(_request, name):
    return Response({"message": f"Hello, {name}!"})


@get("/count")
def count_handler(request):
    app_data = request.app_data
    app_data.counter += 1
    return {"count": app_data.counter}


@get("/query")
def query_handler(request):
    query_params = request.query()
    param = query_params.get("param", "default")
    return {"param": param}


@post("/form")
def form(request):
    input_form = request.form
    return {"username": input_form["username"], "email": input_form["email"]}


@get("/protected")
def protected_handler(request):
    return f"Hello, {request.user_name}!"


def main(static_dir: Path):
    (
        HttpServer(("127.0.0.1", 9999))
        .app_data(AppState())
        .attach(
            Router("/api/v1")
            .route(get("/ping", lambda _: {"message": "pong"}))
            .route(post("/echo", lambda r: {"echo": r.json()}))
            .routes(
                [
                    count_handler,
                    form,
                    hello,
                    query_handler,
                    static_file("/static", str(static_dir)),
                ]
            )
            .scope()
            .middleware(auth_middleware)
            .route(protected_handler)
        )
        .run()
    )


@pytest.fixture(scope="session")
def static_files_dir(tmp_path_factory):
    static_dir = tmp_path_factory.mktemp("static_test")
    (static_dir / "index.html").write_text("<h1>Hello from static file</h1>")
    return static_dir


@pytest.fixture(scope="session")
def oxapy_server(static_files_dir):
    """Run a mock Oxapy HTTP server for integration tests."""
    thread = threading.Thread(target=lambda: main(static_files_dir), daemon=True)
    thread.start()
    time.sleep(2)
    yield "http://127.0.0.1:9999"
