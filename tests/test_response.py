import pytest
from oxapy import Response, Redirect, Status


def test_response_basic():
    resp = Response("Hello, World!")
    assert resp.status == Status.OK


def test_response_with_content_type():
    resp = Response("Hello", content_type="text/plain")
    assert resp.status == Status.OK


def test_response_with_status_enum():
    resp = Response("Error", status=Status.INTERNAL_SERVER_ERROR)
    assert resp.status == Status.INTERNAL_SERVER_ERROR


def test_redirect_default_status():
    redirect = Redirect("/home")
    assert redirect.status == Status.MOVED_PERMANENTLY


def test_redirect_creation():
    redirect = Redirect("/new-location")
    assert redirect is not None
