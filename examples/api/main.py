from utils import hash_password, create_jwt, check_password
from middlewares import jwt_middleware, logger


from sqlalchemy import create_engine
from sqlalchemy.orm import Mapped, mapped_column, DeclarativeBase, Session
from sqlalchemy import String


from oxapy import (
    HttpServer,
    Response,
    Router,
    Status,
    Request,
    serializer,
    get,
    post,
    static_file,
)

import uuid


class AppData:
    def __init__(self):
        self.engine = create_engine("sqlite:///database.db")
        self.n = 0
        Base.metadata.create_all(self.engine)


class Base(DeclarativeBase):
    pass


class User(Base):
    __tablename__ = "users"

    id: Mapped[str] = mapped_column(primary_key=True)
    email: Mapped[str] = mapped_column(
        String(255),
        primary_key=True,
        unique=True,
    )
    password: Mapped[str] = mapped_column(String(255), nullable=False)


class UserSerialzer(serializer.Serializer):
    id = serializer.CharField()
    email = serializer.EmailField()


class UserInputSerializer(serializer.Serializer):
    email = serializer.EmailField()
    password = serializer.CharField()


@post("/register")
def register(request: Request):
    new_user = UserInputSerializer(request)

    try:
        new_user.validate()
    except Exception as e:
        return str(e), Status.BAD_REQUEST

    password = new_user.validate_data["password"]
    email = new_user.validate_data["email"]

    hashed_password = hash_password(password)

    with Session(request.app_data.engine) as session:
        if not session.query(User).filter_by(email=email).first():
            user = User(id=str(uuid.uuid4()), email=email, password=hashed_password)
            session.add(user)
            session.commit()
            return Status.CREATED
        return Status.CONFLICT


@post("/login")
def login(request: Request):
    user_input = UserInputSerializer(request)

    try:
        user_input.validate()
    except Exception as e:
        return str(e), Status.BAD_REQUEST

    email = user_input.validate_data["email"]
    password = user_input.validate_data["password"]

    with Session(request.app_data.engine) as session:
        user = session.query(User).filter_by(email=email).first()
        if user and check_password(user.password, password):
            token = create_jwt(user_id=user.id)
            return {"token": token}
        return Status.UNAUTHORIZED


@get("/hello/{name}")
def hello_world(request: Request, name: str):
    return f"Hello {name}"


@get("/add")
def add(request: Request):
    app_data = request.app_data
    app_data.n += 1
    return app_data.n


@get("/me")
def user_info(request: Request) -> Response:
    with Session(request.app_data.engine) as session:
        if user := session.query(User).filter_by(id=request.user_id).first():
            serializer = UserSerialzer(instance=user)
            return serializer.data


@get("/all")
def all(request: Request) -> Response:
    with Session(request.app_data.engine) as session:
        if user := session.query(User).all():
            serializer = UserSerialzer(instance=user, many=True)
            return serializer.data


pub_router = Router()
pub_router.routes([hello_world, login, register, add])
pub_router.middleware(logger)
pub_router.route(static_file("./static", "static"))

sec_router = Router()
sec_router.routes([user_info, all])
sec_router.middleware(jwt_middleware)
sec_router.middleware(logger)

server = HttpServer(("127.0.0.1", 5555))
server.app_data(AppData())
server.attach(sec_router)
server.attach(pub_router)

if __name__ == "__main__":
    server.run()
