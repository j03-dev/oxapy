import time
from oxapy import Response, serializer  # type: ignore


def test_response_benchmark():
    start = time.perf_counter()
    res = Response({"message": "ok"})
    end = time.perf_counter()
    assert end - start < 0.00003
    assert res.body == '{"message":"ok"}'
