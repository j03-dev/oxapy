from oxapy import serializer, SessionStore, Response, jwt  # type: ignore
import pytest  # type: ignore


def test_serializer():
    class Cred(serializer.Serializer):
        email = serializer.EmailField()
        password = serializer.CharField(min_length=8)

    cred_serializer = Cred(
        '{"email": "test@gmail.com", "password": "password"}'  # type: ignore
    )
    schema = cred_serializer.schema()
    assert schema == {
        "additionalProperties": False,
        "properties": {
            "email": {"format": "email", "type": "string"},
            "password": {"minLength": 8, "type": "string"},
        },
        "required": ["email", "password"],
        "type": "object",
    }

    cred_serializer.is_valid()
    assert cred_serializer.validate_data["email"] == "test@gmail.com"
    assert cred_serializer.validate_data["password"] == "password"

    with pytest.raises(serializer.ValidationException):
        cred_serializer.raw_data = '{"email": "test", "password": "password"}'
        cred_serializer.is_valid()


def test_nested_serializer():
    class Dog(serializer.Serializer):
        name = serializer.CharField()

    class User(serializer.Serializer):
        email = serializer.EmailField()
        password = serializer.CharField(min_length=8)
        dog = Dog()

    nested_serializer = User(
        '{"email": "test@gmail.com", "password": "password", "dog" :{"name": "boby"}}'  # type: ignore
    )

    assert nested_serializer.schema() == {
        "additionalProperties": False,
        "properties": {
            "email": {"format": "email", "type": "string"},
            "password": {"minLength": 8, "type": "string"},
            "dog": {
                "additionalProperties": False,
                "properties": {
                    "name": {"type": "string"},
                },
                "required": ["name"],
                "type": "object",
            },
        },
        "required": ["dog", "email", "password"],
        "type": "object",
    }

    nested_serializer.is_valid()


def test_session_store_usage():
    session_store = SessionStore(
        cookie_name="secure_session",
        cookie_secure=True,
        cookie_same_site="Lax",
    )

    session = session_store.get_session(None)
    session["is_auth"] = True
    assert session["is_auth"]


def test_jwt_generate_and_verify():
    jsonwebtoken = jwt.Jwt("secret")
    token = jsonwebtoken.generate_token({"exp": 60, "sub": "test@gmail.com"})
    claims = jsonwebtoken.verify_token(token)
    assert claims["sub"] == "test@gmail.com"


def test_mult_cookie():
    response = Response("test")
    response.insert_header("Set-Cookie", "userId=abcd123;Path=/")
    response.append_header("Set-Cookie", "theme=dark;Path=/")

    assert response.headers == [
        ("content-type", "application/json"),
        ("set-cookie", "userId=abcd123;Path=/"),
        ("set-cookie", "theme=dark;Path=/"),
    ]
