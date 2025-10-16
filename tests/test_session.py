from oxapy import SessionStore, jwt  # type: ignore


def test_session_store():
    store = SessionStore(cookie_name="secure_session", cookie_secure=True)
    session = store.get_session(None)
    session["is_auth"] = True
    assert session["is_auth"]


def test_jwt_generate_and_verify():
    jsonwebtoken = jwt.Jwt("secret")
    token = jsonwebtoken.generate_token({"exp": 60, "sub": "joe"})
    claims = jsonwebtoken.verify_token(token)
    assert claims["sub"] == "joe"
