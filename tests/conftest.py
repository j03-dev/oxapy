import threading
import time
import pytest
from oxapy import HttpServer, Router, Request

router = Router()


@router.get("/ping")
def ping(_):
    return {"message": "pong"}


@router.post("/echo")
def echo(request: Request):
    return {"echo": request.json()}


@pytest.fixture(scope="session")
def oxapy_server():
    """Run a mock Oxapy HTTP server for integration tests."""
    server = HttpServer(("127.0.0.1", 9999))
    server.attach(router)

    thread = threading.Thread(target=server.run, daemon=True)
    thread.start()

    time.sleep(0.5)

    yield "http://127.0.0.1:9999"
