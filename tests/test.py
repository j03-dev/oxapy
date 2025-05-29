from oxapy import HttpServer, Router, Request

router = Router()


@router.get("/hello/{name:str}")
def greet(request: Request, name: str):
    return f"Hello, {name}!"


app = HttpServer(("127.0.0.1", 5555))
app.attach(router)
app.run()
