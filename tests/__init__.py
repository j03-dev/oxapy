from oxapy import Response


def test_multiple_cookies():
    res = Response("ok")
    res.set_cookie("userId", "123")
    res.set_cookie("theme", "dark")
    assert len([h for h in res.headers if h[0] == "set-cookie"]) == 2
