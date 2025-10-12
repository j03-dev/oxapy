from oxapy import serializer  # type: ignore
import pytest


def test_serializer_basic():
    class Cred(serializer.Serializer):
        email = serializer.EmailField()
        password = serializer.CharField(min_length=8)

    cred = Cred('{"email": "test@gmail.com", "password": "password"}')
    cred.is_valid()
    assert cred.validated_data["email"] == "test@gmail.com"

    with pytest.raises(serializer.ValidationException):
        cred.raw_data = '{"email": "invalid", "password": "password"}'
        cred.is_valid()


def test_nested_serializer():
    class Dog(serializer.Serializer):
        name = serializer.CharField()
        toys = serializer.CharField(many=True, nullable=True)

    class User(serializer.Serializer):
        email = serializer.EmailField()
        password = serializer.CharField(min_length=8)
        dog = Dog(nullable=True)  # type: ignore

    user = User(
        '{"email": "test@gmail.com", "password": "password", "dog":{"name":"boby","toys":null}}'
    )
    user.is_valid()


def test_read_write_only_fields():
    class UserSerializer(serializer.Serializer):
        id = serializer.CharField(read_only=True, nullable=True, required=False)  # type: ignore
        name = serializer.CharField()
        password = serializer.CharField(write_only=True)  # type: ignore

    user_serializer = UserSerializer(
        '{"id": null, "name": "joe", "password": "password"}'
    )
    user_serializer.is_valid()
    assert user_serializer.validated_data == {"name": "joe", "password": "password"}
