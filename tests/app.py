from oxapy import HttpServer, Router, get


@get("/greet/{name}")
def greet(_r, name: str):
    return f"Hello, {name}!"


HttpServer(("0.0.0.0", 5555)).attach(Router().route(greet)).run()
