from oxapy import serializer, SessionStore, Response, jwt  # type: ignore
from oxapy import exceptions  # type: ignore
from sqlalchemy.orm import Mapped, mapped_column, DeclarativeBase, relationship  # type: ignore
from sqlalchemy import ForeignKey  # type: ignore
import pytest  # type: ignore
import time


class Base(DeclarativeBase):
    pass


class Dog(Base):
    __tablename__ = "dogs"

    id: Mapped[str] = mapped_column(primary_key=True)
    name: Mapped[str] = mapped_column(nullable=False)
    owner: Mapped[str] = mapped_column(ForeignKey("users.id"))

    user: Mapped["User"] = relationship(back_populates="dog")


class User(Base):
    __tablename__ = "users"

    id: Mapped[str] = mapped_column(primary_key=True)
    name: Mapped[str] = mapped_column(nullable=False)
    password: Mapped[str] = mapped_column(nullable=False)

    dog: Mapped["Dog"] = relationship(back_populates="user", uselist=False)


def test_serializer():
    class Cred(serializer.Serializer):
        email = serializer.EmailField()
        password = serializer.CharField(min_length=8)

    cred_serializer = Cred('{"email": "test@gmail.com", "password": "password"}')
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
    assert cred_serializer.validated_data
    assert cred_serializer.validated_data["email"] == "test@gmail.com"
    assert cred_serializer.validated_data["password"] == "password"

    with pytest.raises(serializer.ValidationException):
        cred_serializer.raw_data = '{"email": "test", "password": "password"}'
        cred_serializer.is_valid()


def test_nested_serializer():
    class Dog(serializer.Serializer):
        name = serializer.CharField()
        toys = serializer.CharField(many=True, nullable=True)

    class User(serializer.Serializer):
        email = serializer.EmailField()
        password = serializer.CharField(min_length=8)
        dog = Dog(nullable=True)  # type: ignore

    nested_serializer = User(
        '{"email": "test@gmail.com", "password": "password", "dog" :{"name": "boby", "toys": null}}'
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
                    "toys": {
                        "items": {"type": ["string", "null"]},
                        "type": ["array", "null"],
                    },
                },
                "required": ["name", "toys"],
                "type": ["object", "null"],
            },
        },
        "required": ["dog", "email", "password"],
        "type": "object",
    }

    nested_serializer.is_valid()


def test_serializer_read_and_write_only():
    class UserSerializer(serializer.Serializer):
        id = serializer.CharField(read_only=True, nullable=True, required=False)  # type: ignore
        name = serializer.CharField()
        password = serializer.CharField(write_only=True)  # type: ignore

    user_serializer = UserSerializer(
        # type: ignore
        '{"id": null, "name": "joe", "password": "password"}'
    )
    user_serializer.is_valid()

    user = User(id="abcd1234", name="joe", password="password")  # type: ignore

    assert user_serializer.validated_data == {"name": "joe", "password": "password"}

    new_user_serializer = UserSerializer(instance=user)  # type: ignore
    assert new_user_serializer.data == {"id": "abcd1234", "name": "joe"}


def test_serializer_pattern():
    class PhoneNumberSerializer(serializer.Serializer):
        phone_number = serializer.CharField(
            format="phone_number", pattern=r"^(?:\+261|0)(32|33|34|37|38)\d{7}$"
        )

    w = PhoneNumberSerializer('{"phone_number":"0325911514"}')
    w.is_valid()


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


def test_serializer_bench():
    class DogSerializer(serializer.Serializer):
        id = serializer.CharField(read_only=True, nullable=True, required=False)  # type: ignore
        name = serializer.CharField()

    class UserSerializer(serializer.Serializer):
        id = serializer.CharField(read_only=True, nullable=True, required=False)  # type: ignore
        name = serializer.CharField()
        password = serializer.CharField(write_only=True)  # type: ignore
        dog = DogSerializer(read_only=True)

    user = User(id="abcd1234", name="joe", password="password")  # type: ignore
    user.dog = Dog(id="efgh5678", name="boby", owner="abcd1234")

    user_serializer = UserSerializer(instance=user)  # type: ignore
    start = time.perf_counter()
    data = user_serializer.data
    end = time.perf_counter()

    assert end - start < 0.0002

    assert data == {
        "id": "abcd1234",
        "name": "joe",
        "dog": {"id": "efgh5678", "name": "boby"},
    }


def test_bench_create_response():
    start = time.perf_counter()
    response = Response({"message": "User is created"})
    end = time.perf_counter()
    assert end - start < 0.00002
    assert response.body == '{"message":"User is created"}'
