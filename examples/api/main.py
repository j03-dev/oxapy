from utils import hash_password, create_jwt, check_password
from middlewares import jwt_middleware, logger


from sqlalchemy import create_engine
from sqlalchemy.orm import Mapped, mapped_column, DeclarativeBase, Session
from sqlalchemy import String

from functools import wraps
from typing import TypeVar, Callable, Any

from oxapy import (
    HttpServer,
    Response,
    Router,
    Status,
    Request,
    serializer,
    get,
    post,
)

import uuid


F = TypeVar("F", bound=Callable[..., Any])


def with_session(func: F) -> F:
    @wraps(func)
    def wrapper(request: Request, *args, **kwargs):
        with Session(request.app_data.engine) as session:
            return func(request, session, *args, **kwargs)

    return wrapper


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


class UserSerializer(serializer.Serializer):
    id = serializer.UUIDField()
    email = serializer.EmailField()


class UserInputSerializer(serializer.Serializer):
    email = serializer.EmailField()
    password = serializer.CharField(min_length=8)

    class Meta:
        model = User


@post("/register")
@with_session
def register(request: Request, session: Session):
    new_user = UserInputSerializer(request)
    new_user.is_valid()

    new_user.validate_data.update(
        {
            "id": str(uuid.uuid4()),
            "password": hash_password(new_user.validate_data["password"]),
        }
    )

    if not session.query(User).filter_by(email=new_user.validate_data["email"]).first():
        new_user.save(session)
        return Status.OK
    return Status.CONFLICT


@post("/login")
@with_session
def login(request: Request, session: Session):
    user_input = UserInputSerializer(request)
    user_input.is_valid()
    email = user_input.validate_data["email"]
    password = user_input.validate_data["password"]

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
@with_session
def user_info(request: Request, session: Session) -> Response:
    if user := session.query(User).filter_by(id=request.user_id).first():
        serializer = UserSerializer(instance=user)
        return serializer.data


@get("/all")
@with_session
def all(request: Request, session: Session) -> Response:
    if user := session.query(User).all():
        serializer = UserSerializer(instance=user, many=True)
        return serializer.data


pub_router = Router()
pub_router.routes([hello_world, login, register, add])
pub_router.middleware(logger)

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
