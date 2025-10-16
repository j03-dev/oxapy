from oxapy import Response


def test_multiple_cookies():
    res = Response("ok")
    res.insert_header("Set-Cookie", "userId=123;Path=/")
    res.append_header("Set-Cookie", "theme=dark;Path=/")
    assert len([h for h in res.headers if h[0] == "set-cookie"]) == 2
