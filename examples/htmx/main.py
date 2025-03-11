from oxapy import templating
from oxapy import static_file, get, post, HttpServer, Status, Router, Response


class AppState:
    def __init__(self):
        self.template = templating.Jinja("./templates/**/*.html.j2")


@get("/")
def index_page(app_data):
    return Response(
        Status.OK,
        app_data.template.render("index.html.j2", {"name": "world"}),
        "text/html",
    )


@get("/login")
def login_page(app_data):
    return Response(
        Status.OK,
        app_data.template.render("login.html.j2"),
        "text/html",
    )


@post("/login", data="cred")
def login_form(cred, app_data):
    username = cred.get("username")
    password = cred.get("password")
    if username == "admin" and password == "password":
        return "Login success", Status.OK
    return Response(
        Status.OK,
        app_data.template.render(
            "error_message.html.j2",
            {"error_message": "login failed: Unauthorized"},
        ),
        "text/html",
    )


def logger(request, next, **kwargs):
    print(f"{request.method} {request.uri}")
    return next(**kwargs)


router = Router()
router.middleware(logger)
router.routes(
    [
        index_page,
        login_page,
        login_form,
        static_file("./static", "static"),
    ]
)


server = HttpServer(("127.0.0.1", 8080))
server.attach(router)
server.app_data(AppState())

if __name__ == "__main__":
    server.run()
