from oxapy import Status
from utils import decode_jwt
import datetime


def jwt_middleware(request, next, **kwargs):
    token = request.headers.get("authorization", "").replace("Bearer ", "")

    if token:
        if payload := decode_jwt(token):
            request.user_id = payload["user_id"]
            return next(request, **kwargs)
    return Status.UNAUTHORIZED


def logger(request, next, **kwargs):
    print(f"[{datetime.datetime.utcnow()}] {request.method} {request.uri}")
    return next(request, **kwargs)
