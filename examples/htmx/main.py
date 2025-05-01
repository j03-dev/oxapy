from oxapy import templating
from oxapy import static_file, get, post, HttpServer, Status, Router, Redirect
from oxapy import serializer, Request, SessionStore, Session


@get("/")
def index_page(request: Request):
    session: Session = request.session()
    if session.get("is_auth"):
        return templating.render(request, "index.html.j2", {"name": "word"})
    return Redirect("/login")


@get("/login")
def login_page(request: Request):
    return templating.render(request, "login.html.j2")


@post("/upload-file")
def upload_file(request: Request):
    if file := request.files().get("file"):
        file.save(f"media/{file.name}")
    return Status.OK


class CredSerializer(serializer.Serializer):
    username = serializer.CharField()
    password = serializer.CharField()


@post("/login")
def login_form(request: Request):
    cred = CredSerializer(request)

    try:
        cred.validate()
    except Exception as e:
        return str(e), Status.OK

    username = cred.validate_data["username"]
    password = cred.validate_data["password"]

    if username == "admin" and password == "password":
        session = request.session()
        session["is_auth"] = True
        return "Login success", Status.OK
    return templating.render(
        request, "components/error_mesage.html.j2", {"error_message": "Login failed"}
    )


router = Router()
router.routes([index_page, login_page, login_form, upload_file])
router.route(static_file("./static", "static"))


server = HttpServer(("127.0.0.1", 8080))
server.session_store(SessionStore())
server.attach(router)
server.template(templating.Template("./templates/**/*.html.j2"))

if __name__ == "__main__":
    server.run()
