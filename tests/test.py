from http import client


conn = client.HTTPConnection("127.0.0.1", 5555)


def test_hello_word():
    conn.request("GET", "/hello/joe")
    response = conn.getresponse()
    assert response.status == 200
    assert response.read(response.length).decode() == "Hello, joe!"


def test_query():
    conn.request("GET", "/query?message=Hello,%20World!")
    response = conn.getresponse()
    assert response.status == 200
    assert response.read(response.length).decode() == "Hello, World!"
