from oxapy import templating
from oxapy import static_file, get, post, HttpServer, Status, Router
from oxapy import serializer


@get("/")
def index_page(request):
    return templating.render(request, "index.html.j2", {"name": "word"})


@get("/login")
def login_page(request):
    return templating.render(request, "login.html.j2")


class CredSerializer(serializer.Serializer):
    username = serializer.Field("string")
    password = serializer.Field("string")


@post("/login")
def login_form(request):
    cred = CredSerializer(request)

    try:
        cred.validate()
    except Exception as e:
        return str(e), Status.OK

    username = cred.validate_data["username"]
    password = cred.validate_data["password"]

    if username == "admin" and password == "password":
        return "Login success", Status.OK
    return templating.render(
        request, "components/error_mesage.html.j2", {"error_message": "Login failed"}
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

template = templating.Template("jinja", "./templates/**/*.html.j2")
server.template(template)

if __name__ == "__main__":
    server.run()
