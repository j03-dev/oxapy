from oxapy import templating
from oxapy import static_file, get, post, HttpServer, Status, Router, Response
from oxapy import serializer


class AppState:
    def __init__(self):
        self.template = templating.Jinja("./templates/**/*.html.j2")


@get("/")
def index_page(request):
    return Response(
        Status.OK,
        request.app_data.template.render("index.html.j2", {"name": "world"}),
        "text/html",
    )


@get("/login")
def login_page(request):
    return Response(
        Status.OK,
        request.app_data.template.render("login.html.j2"),
        "text/html",
    )


class CredSerializer(serializer.Serializer):
    username = serializer.Field("string")
    password = serializer.Field("string")


@post("/login")
def login_form(request):
    cred = CredSerializer()

    try:
        cred.validate()
    except Exception as e:
        return str(e), Status.BAD_REQUEST

    username = cred.validate_data["username"]
    password = cred.validate_data["password"]

    if username == "admin" and password == "password":
        return "Login success", Status.OK
    return Response(
        Status.OK,
        request.app_data.template.render(
            "components/error_message.html.j2",
            {"error_message": "login failed: Unauthorized"},
        ),
        "text/html",
    )


def logger(request, next, **kwargs):
    print(f"{request.method} {request.uri}")
    return next(request, **kwargs)


router = Router()
router.middleware(logger)
router.routes([index_page, login_page, login_form])
router.route(static_file("./static", "static"))


server = HttpServer(("127.0.0.1", 8080))
server.attach(router)
server.app_data(AppState())

if __name__ == "__main__":
    server.run()
