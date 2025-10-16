import requests


def test_ping_endpoint(oxapy_server):
    res = requests.get(f"{oxapy_server}/ping")
    assert res.status_code == 200
    assert res.json()["message"] == "pong"


def test_echo_endpoint(oxapy_server):
    payload = {"msg": "hello"}
    res = requests.post(f"{oxapy_server}/echo", json=payload)
    assert res.status_code == 200
    assert res.json()["echo"] == payload
