from oxapy import HttpServer, Router, Request


router = Router()


@router.get("/hello/{name}")
def greet(request: Request, name: str):
    return f"Hello , {name}!"


app = HttpServer(("0.0.0.0", 5555))
app.attach(router)
app.run()
