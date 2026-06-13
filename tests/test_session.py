import pytest
from oxapy import jwt


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
