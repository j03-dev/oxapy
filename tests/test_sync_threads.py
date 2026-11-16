import socket
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import pytest
import requests


def _free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def _start_server(threads):
    port = _free_port()
    proc = subprocess.Popen(
        [
            sys.executable,
            str(Path(__file__).with_name("sync_threaded_app.py")),
            str(port),
            str(threads),
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    base_url = f"http://127.0.0.1:{port}"
    deadline = time.time() + 5
    try:
        while time.time() < deadline:
            if proc.poll() is not None:
                pytest.fail(f"sync server (threads={threads}) exited during startup")
            try:
                if requests.get(f"{base_url}/health", timeout=0.2).status_code == 200:
                    return proc, base_url
            except requests.RequestException:
                pass
            time.sleep(0.05)
        pytest.fail(f"sync server (threads={threads}) did not start")
    except BaseException:
        _stop(proc)
        raise


def _stop(proc):
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()


@pytest.fixture(scope="module", params=[1, 4], ids=["serial", "threaded"])
def sync_server(request):
    proc, base_url = _start_server(request.param)
    try:
        yield request.param, base_url
    finally:
        _stop(proc)


def test_sync_handler_concurrency_matches_thread_count(sync_server):
    threads, base_url = sync_server
    request_count = 8

    def hit(_):
        return requests.get(f"{base_url}/slow", timeout=15)

    start = time.perf_counter()
    with ThreadPoolExecutor(max_workers=request_count) as pool:
        responses = list(pool.map(hit, range(request_count)))
    elapsed = time.perf_counter() - start

    assert all(response.status_code == 200 for response in responses)
    if threads == 1:
        assert elapsed >= 1.8, f"serial handlers finished too fast: {elapsed:.2f}s"
    else:
        assert elapsed < 1.8, f"{threads} handler threads took {elapsed:.2f}s"
