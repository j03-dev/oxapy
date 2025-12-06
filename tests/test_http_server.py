import requests
from tests.utils import Multipart


def test_ping_endpoint(oxapy_server):
    res = requests.get(f"{oxapy_server}/api/v1/ping")
    assert res.status_code == 200
    assert res.json()["message"] == "pong"
    assert "application/json" in res.headers["Content-Type"]


def test_echo_endpoint(oxapy_server):
    payload = {"msg": "hello"}
    res = requests.post(f"{oxapy_server}/api/v1/echo", json=payload)
    assert res.status_code == 200
    assert res.json()["echo"] == payload
    assert "application/json" in res.headers["Content-Type"]


def test_path_parameter_endpoint(oxapy_server):
    name = "World"
    res = requests.get(f"{oxapy_server}/api/v1/hello/{name}")
    assert res.status_code == 200
    assert res.json()["message"] == f"Hello, {name}!"
    assert "application/json" in res.headers["Content-Type"]


def test_app_state_endpoint(oxapy_server):
    res1 = requests.get(f"{oxapy_server}/api/v1/count")
    assert res1.status_code == 200
    count1 = res1.json()["count"]

    res2 = requests.get(f"{oxapy_server}/api/v1/count")
    assert res2.status_code == 200
    count2 = res2.json()["count"]
    assert count2 == count1 + 1
    assert "application/json" in res2.headers["Content-Type"]


def test_query_string_endpoint(oxapy_server):
    param = "test_param"
    res = requests.get(f"{oxapy_server}/api/v1/query?param={param}")
    assert res.status_code == 200
    assert res.json()["param"] == param
    assert "application/json" in res.headers["Content-Type"]

    res_default = requests.get(f"{oxapy_server}/api/v1/query")
    assert res_default.status_code == 200
    assert res_default.json()["param"] == "default"


def test_static_file_serving(oxapy_server):
    res = requests.get(f"{oxapy_server}/api/v1/static/index.html")
    assert res.status_code == 200
    assert res.text == "<h1>Hello from static file</h1>"
    assert "text/html" in res.headers["Content-Type"]


def test_static_file_not_found(oxapy_server):
    res = requests.get(f"{oxapy_server}/api/v1/static/nonexistent.html")
    assert res.status_code == 404


def test_form(oxapy_server):
    form_data = {"username": "John Does", "email": "johndoes@email.com"}
    multipart = Multipart(form_data)
    res = requests.post(
        f"{oxapy_server}/api/v1/form", data=multipart.data, headers=multipart.headers
    )
    assert res.status_code == 200
    assert res.json() == form_data


def test_protected_route_unauthorized(oxapy_server):
    res = requests.get(f"{oxapy_server}/api/v1/protected")
    assert res.status_code == 401


def test_protected_route_authorized(oxapy_server):
    headers = {"Authorization": "Bearer some_token"}
    res = requests.get(f"{oxapy_server}/api/v1/protected", headers=headers)
    assert res.status_code == 200
    assert res.text == "Hello, John Does!"


def test_not_found_endpoint(oxapy_server):
    res = requests.get(f"{oxapy_server}/api/v1/nonexistent")
    assert res.status_code == 404


def test_method_not_allowed_returns_404(oxapy_server):
    res = requests.get(f"{oxapy_server}/api/v1/echo")
    assert res.status_code == 404
