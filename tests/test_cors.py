import pytest
from oxapy import Cors


def test_cors_default_values():
    cors = Cors()
    assert cors.origins == ["*"]
    assert cors.allow_credentials is True
    assert cors.max_age == 86400


def test_cors_set_origins():
    cors = Cors()
    cors.origins = ["http://example.com", "https://app.example.com"]
    assert cors.origins == ["http://example.com", "https://app.example.com"]


def test_cors_set_methods():
    cors = Cors()
    cors.methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
    assert "GET" in cors.methods
    assert "POST" in cors.methods
    assert "DELETE" in cors.methods


def test_cors_set_headers():
    cors = Cors()
    cors.headers = ["Content-Type", "Authorization", "X-Custom-Header"]
    assert "Content-Type" in cors.headers
    assert "Authorization" in cors.headers


def test_cors_credentials():
    cors = Cors()
    cors.allow_credentials = False
    assert cors.allow_credentials is False


def test_cors_max_age():
    cors = Cors()
    cors.max_age = 3600
    assert cors.max_age == 3600
