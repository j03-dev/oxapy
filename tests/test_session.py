import pytest
from oxapy import SessionStore, jwt


def test_session_store():
    store = SessionStore(cookie_name="secure_session", cookie_secure=True)
    session = store.get_session(None)
    session["is_auth"] = True
    assert session["is_auth"]


def test_session_store_multiple_keys():
    store = SessionStore()
    session = store.get_session(None)
    session["user_id"] = 123
    session["username"] = "john"
    session["roles"] = ["admin", "user"]
    assert session["user_id"] == 123
    assert session["username"] == "john"
    assert session["roles"] == ["admin", "user"]


def test_jwt_generate_and_verify():
    jsonwebtoken = jwt.Jwt("secret")
    token = jsonwebtoken.generate_token({"exp": 60, "sub": "joe"})
    claims = jsonwebtoken.verify_token(token)
    assert claims["sub"] == "joe"


def test_jwt_with_custom_claims():
    jsonwebtoken = jwt.Jwt("mysecret")
    token = jsonwebtoken.generate_token(
        {"sub": "user123", "role": "admin", "permissions": ["read", "write"]}
    )
    claims = jsonwebtoken.verify_token(token)
    assert claims["sub"] == "user123"
    assert claims["role"] == "admin"


def test_jwt_invalid_token():
    jsonwebtoken = jwt.Jwt("secret")
    with pytest.raises(Exception):
        jsonwebtoken.verify_token("invalid.token.here")
