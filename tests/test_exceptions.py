import pytest
from oxapy import exceptions


def test_bad_request_exception():
    exc = exceptions.BadRequestError("Invalid input")
    assert str(exc) == "Invalid input"


def test_unauthorized_exception():
    exc = exceptions.UnauthorizedError("Not authenticated")
    assert str(exc) == "Not authenticated"


def test_forbidden_exception():
    exc = exceptions.ForbiddenError("Access denied")
    assert str(exc) == "Access denied"


def test_not_found_exception():
    exc = exceptions.NotFoundError("Resource not found")
    assert str(exc) == "Resource not found"


def test_client_error_is_base():
    exc = exceptions.ClientError("Generic client error")
    assert isinstance(exc, Exception)
