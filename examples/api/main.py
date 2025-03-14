import sqlite3
from utils import hash_password, create_jwt, check_password
from middlewares import jwt_middleware, logger

from oxapy import (
    HttpServer,
    Response,
    Router,
    Status,
    get,
    post,
    static_file,
)


@post("/register")
def register(request):
    conn = request.app_data.conn

    data = request.json()
    username = data.get("username")
    password = data.get("password")

    if not username or not password:
        return Status.BAD_REQUEST

    hashed_password = hash_password(password)

    try:
        conn.execute(
            "insert into user (username, password) values (?, ?)",
            (username, hashed_password),
        )
        conn.commit()
        return Status.CREATED
    except sqlite3.IntegrityError:
        return Status.CONFLICT


@post("/login")
def login(request):
    conn = request.app_data.conn

    data = request.json()
    username = data.get("username")
    password = data.get("password")

    cursor = conn.execute(
        "select id, password from user where username=?",
        (username,),
    )
    user = cursor.fetchone()

    if user and check_password(user[1], password):
        token = create_jwt(user_id=user[0])
        return {"token": token}

    return Status.UNAUTHORIZED


@get("/hello/{name}")
def hello_world(request, name):
    return f"Hello {name}"


@get("/add")
def add(request):
    app_data = request.app_data
    app_data.n += 1
    return app_data.n


@get("/me")
def user_info(request, user_id: int) -> Response:
    app_data = request.app_data
    result = app_data.conn.execute("select * from user where id=?", (user_id,))
    return Response(Status.OK, {"user": result.fetchone()})


class AppData:
    def __init__(self):
        self.conn = sqlite3.connect("database.db")
        self.n = 0


pub_router = Router()
pub_router.routes([hello_world, login, register, add])
pub_router.middleware(logger)
pub_router.route(static_file("./static", "static"))

sec_router = Router()
sec_router.route(user_info)
sec_router.middleware(jwt_middleware)
sec_router.middleware(logger)

server = HttpServer(("127.0.0.1", 5555))
server.app_data(AppData())
server.attach(sec_router)
server.attach(pub_router)

if __name__ == "__main__":
    server.run()
