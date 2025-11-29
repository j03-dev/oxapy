import threading
import time
import pytest
from oxapy import HttpServer, Router, get, post


def main():
    (
        HttpServer(("127.0.0.1", 9999))
        .attach(
            Router("/api/v1")
            .route(get("/ping", lambda _: {"message": "pong"}))
            .route(post("/echo", lambda r: {"echo": r.json()}))
        )
        .run()
    )


@pytest.fixture(scope="session")
def oxapy_server():
    """Run a mock Oxapy HTTP server for integration tests."""
    thread = threading.Thread(target=main, daemon=True)
    thread.start()
    time.sleep(2)
    yield "http://127.0.0.1:9999"
