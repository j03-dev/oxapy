import threading
import time
import pytest
from oxapy import HttpServer, Router, Request, get, post
import logging


def log(r, next, **kwargs):
    logging.log(1000, f"{r.method} {r.uri}")
    return next(r, **kwargs)


router = (
    Router("/api/v1")
    .route(get("/ping", lambda _: {"message": "pong"}))
    .route(post("/echo", lambda r: {"echo": r.json()}))
    .service()
    .middleware(log)
    .route(get("/hello/{name}", lambda _, name: f"Hello, {name}"))
)


def main():
    server = HttpServer(("127.0.0.1", 9999))
    server.attach(router)
    server.run()


@pytest.fixture(scope="session")
def oxapy_server():
    """Run a mock Oxapy HTTP server for integration tests."""
    thread = threading.Thread(target=main, daemon=True)
    thread.start()
    time.sleep(2)
    yield "http://127.0.0.1:9999"
