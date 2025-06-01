from oxapy import HttpServer, Router, Request, Status

import logging


def logger(request, next, **kwargs):
    logging.log(1000, f"{request.method} {request.uri}")
    return next(request, **kwargs)


router = Router()
router.middleware(logger)


@router.get("/hello/{name}")
def greet(request: Request, name: str):
    return f"Hello, {name}!"


@router.get("/query")
def query(request: Request):
    if message := request.query():
        try:
            return message.get("message")
        except Exception as e:
            return str(e)
    return Status.BAD_REQUEST


app = HttpServer(("0.0.0.0", 5555))
app.attach(router)
app.run()
