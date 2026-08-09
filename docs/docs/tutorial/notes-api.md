# Build a Notes API

This tutorial builds a complete, runnable **Notes API** with OxAPY, the way the library is used in production:

- SQLAlchemy models and a database session middleware
- Serializers for validation and persistence
- JWT authentication as middleware
- Async handlers in async mode
- Typed path parameters, query strings, and JSON bodies

Along the way, every feature links to the guide or API page that covers it in depth.

## What we are building

A small REST API where users sign up, log in, and manage personal notes:

| Method | Path | Auth | Purpose |
| --- | --- | --- | --- |
| `POST` | `/api/signup` | no | Create an account |
| `POST` | `/api/signin` | no | Log in, get a JWT |
| `GET` | `/api/me` | yes | Current user |
| `GET` | `/api/notes` | yes | List my notes |
| `POST` | `/api/notes` | yes | Create a note |
| `GET` | `/api/notes/{note_id}` | yes | Read one note |
| `DELETE` | `/api/notes/{note_id}` | yes | Delete a note |

## Project layout

```
notes_api/
├── app.py          # server wiring
├── config.py       # settings
├── models.py       # SQLAlchemy models
├── db.py           # engine + per-request session
├── serializers.py  # OxAPY serializers
├── middlewares.py  # db session + JWT auth
├── services.py     # business logic
└── routes.py       # route handlers
```

## 1. Configuration

`config.py` — keep settings in one place. Secrets come from the environment:

```python
import os

HOST = "127.0.0.1"
PORT = 8000
DATABASE_URL = os.getenv("DATABASE_URL", "sqlite:///notes.db")
SECRET_KEY = os.getenv("SECRET_KEY", "dev-secret-change-me")
```

:::warning

The `SECRET_KEY` signs your JWTs. In production, always set it via an environment variable and use a long random value.

:::

## 2. Models

`models.py` — plain SQLAlchemy models. Note how `User` and `Note` are related: a user owns many notes.

```python
from sqlalchemy import ForeignKey, String, Text
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, relationship
from sqlalchemy.types import DateTime
from datetime import datetime


class Base(DeclarativeBase):
    pass


class User(Base):
    __tablename__ = "users"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    email: Mapped[str] = mapped_column(String(255), unique=True)
    full_name: Mapped[str] = mapped_column(String(255))
    password_hash: Mapped[str] = mapped_column(String(255))
    created_at: Mapped[datetime] = mapped_column(DateTime, default=datetime.utcnow)

    notes: Mapped[list["Note"]] = relationship(back_populates="user")


class Note(Base):
    __tablename__ = "notes"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    title: Mapped[str] = mapped_column(String(255))
    content: Mapped[str | None] = mapped_column(Text, nullable=True)
    user_id: Mapped[str] = mapped_column(ForeignKey("users.id"))
    created_at: Mapped[datetime] = mapped_column(DateTime, default=datetime.utcnow)

    user: Mapped[User] = relationship(back_populates="notes")
```

## 3. Database session

A small module gives every handler a session. We create the engine once and expose a context manager that yields a session per request:

```python
# db.py
from sqlalchemy import create_engine
from sqlalchemy.orm import Session, sessionmaker

from .config import DATABASE_URL
from .models import Base

engine = create_engine(DATABASE_URL, connect_args={"check_same_thread": False})
SessionLocal = sessionmaker(bind=engine, class_=Session)


def init_db():
    Base.metadata.create_all(engine)


class DB:
    def __enter__(self):
        self.session = SessionLocal()
        return self.session

    def __exit__(self, *args):
        self.session.close()
```

## 4. Serializers

`serializers.py` — this is where OxAPY serializers shine. They validate input and persist model instances.

```python
from uuid import uuid4

from oxapy import serializer

from .models import Note, User


def new_id() -> str:
    return str(uuid4())


class UserSerializer(serializer.Serializer):
    id = serializer.CharField(read_only=True, required=False, nullable=True)
    email = serializer.EmailField()
    full_name = serializer.CharField()
    password = serializer.CharField(min_length=8, write_only=True)
    created_at = serializer.DateTimeField(required=False, nullable=True, read_only=True)

    class Meta:
        model = User

    def create(self, session, validated_data):
        validated_data["id"] = new_id()
        return super().create(session, validated_data)


class NoteSerializer(serializer.Serializer):
    id = serializer.CharField(required=False, read_only=True)
    title = serializer.CharField()
    content = serializer.CharField(nullable=True, required=False)
    user_id = serializer.CharField(required=False, read_only=True)
    created_at = serializer.DateTimeField(required=False, read_only=True)

    class Meta:
        model = Note

    def create(self, session, validated_data):
        validated_data["id"] = new_id()
        validated_data["user_id"] = self.context["user_id"]
        return super().create(session, validated_data)
```

What happens here:

- `class Meta: model = User` tells the base `create()` how to build and persist the row (`Model(**validated_data)`, then `session.add/commit/refresh`).
- `password` is `write_only=True`, so it is validated and stored but never appears in serialized output. `id` and `created_at` are `read_only=True`, so clients cannot submit them.
- The overridden `create()` adds an `id` (and, for notes, the owning user from `context`) before delegating to `super().create(...)`.
- `content` is `nullable=True` and `required=False` because a note body is optional.

See the [Serializers guide](../guides/serializers) for the full picture.

## 5. Middleware

`middlewares.py` — two pieces of middleware, exactly as you would write them in a real app:

```python
from typing import Callable

from oxapy import Request, Response, Status, jwt

from .config import SECRET_KEY
from .db import DB

Next = Callable[[Request], Response]

JWT = jwt.Jwt(secret=SECRET_KEY, algorithm="HS256")


def db(req: Request, next: Next, **kwargs) -> Response:
    with DB() as _db:
        req.db = _db
        return next(req, **kwargs)


def auth(req: Request, next: Next, **kwargs) -> Response:
    token = req.headers.get("Authorization", "").replace("Bearer ", "")
    try:
        claims = JWT.verify_token(token)
        req.user_id = claims["sub"]
    except jwt.JwtError:
        return Status.UNAUTHORIZED
    return next(req, **kwargs)
```

- `db` opens a session per request, attaches it as `request.db`, and closes it after the handler runs.
- `auth` verifies the `Authorization: Bearer <token>` header and stores `request.user_id`. A missing or invalid token short-circuits with `401 Unauthorized`.

Both stay synchronous even in async mode — only handlers need to be `async def`.

See the [Middleware guide](../guides/middleware) and [JWT guide](../guides/jwt-authentication).

## 6. Services

`services.py` — business logic separated from HTTP concerns. Errors are raised as OxAPY exceptions so the framework builds the response:

```python
import hashlib
import hmac

from sqlalchemy.orm import Session

from oxapy import exceptions

from .models import Note, User
from .serializers import NoteSerializer, UserSerializer


def hash_password(password: str) -> str:
    salt = hmac.new(b"oxapy", password.encode(), hashlib.sha256).hexdigest()
    return hmac.new(salt.encode(), password.encode(), hashlib.sha256).hexdigest()


def verify_password(password: str, password_hash: str) -> bool:
    return hmac.compare_digest(hash_password(password), password_hash)


def register(db: Session, new_user: UserSerializer) -> User:
    try:
        validated = new_user.validated_data
        validated["password_hash"] = hash_password(validated.pop("password"))
        return new_user.save(db)
    except Exception as e:
        raise exceptions.BadRequestError(f"Registration failed: {e}")


def login(db: Session, email: str, password: str) -> User:
    user = db.query(User).filter(User.email == email).first()
    if user is None or not verify_password(password, user.password_hash):
        raise exceptions.UnauthorizedError("Invalid credentials")
    return user


def get_user_by_id(db: Session, user_id: str) -> User:
    user = db.query(User).filter(User.id == user_id).first()
    if user is None:
        raise exceptions.NotFoundError(f"User with ID {user_id} is not found")
    return user


def list_notes(db: Session, user_id: str):
    return db.query(Note).filter(Note.user_id == user_id).order_by(Note.created_at.desc()).all()


def get_note(db: Session, user_id: str, note_id: str) -> Note:
    note = db.query(Note).filter(Note.id == note_id, Note.user_id == user_id).first()
    if note is None:
        raise exceptions.NotFoundError("Note not found")
    return note
```

:::note

The password helpers use Python's stdlib `hmac`/`hashlib` to keep the tutorial self-contained. In production, use a dedicated password-hashing library such as `argon2-cffi` or `bcrypt`.

:::

## 7. Routes

`routes.py` — thin handlers that read the request, call services, and return data. Note the `(body, Status)` tuples and async handlers.

```python
from oxapy import (
    Request,
    Status,
    delete,
    get,
    post,
)

from .middlewares import JWT
from .serializers import NoteSerializer, UserSerializer
from .services import get_note, get_user_by_id, list_notes, login, register


@post("/signup")
async def signup(req: Request):
    new_user = UserSerializer(req.data)
    new_user.is_valid()
    user = register(req.db, new_user)
    return {"user": UserSerializer(instance=user).data}, Status.CREATED


@post("/signin")
async def signin(req: Request):
    body = req.json()
    user = login(req.db, body["email"], body["password"])
    token = JWT.generate_token({"sub": user.id, "exp": 3600 * 24 * 7})  # 1 week
    return {"user": UserSerializer(instance=user).data, "token": token}, Status.ACCEPTED


@get("/me")
async def me(req: Request):
    user = get_user_by_id(req.db, req.user_id)
    return {"me": UserSerializer(instance=user).data}


@get("/notes")
async def notes(req: Request):
    user_notes = list_notes(req.db, req.user_id)
    return {"notes": NoteSerializer(instance=user_notes, many=True).data}


@post("/notes")
async def create_note(req: Request):
    new_note = NoteSerializer(req.data, context={"user_id": req.user_id})
    new_note.is_valid()
    note = new_note.save(req.db)
    return {"note": NoteSerializer(instance=note).data}, Status.CREATED


@get("/notes/{note_id}")
async def note_detail(req: Request, note_id: str):
    note = get_note(req.db, req.user_id, note_id)
    return {"note": NoteSerializer(instance=note).data}


@delete("/notes/{note_id}")
async def delete_note(req: Request, note_id: str):
    note = get_note(req.db, req.user_id, note_id)
    req.db.delete(note)
    req.db.commit()
    return Status.OK
```

A few details worth copying:

- `UserSerializer(req.data)` takes the raw JSON body; `is_valid()` populates `validated_data` and raises `serializer.ValidationException` on bad input.
- `NoteSerializer(req.data, context={"user_id": req.user_id})` passes the acting user, which the overridden `create()` reads from `self.context`.
- `{note_id}` captures the UUID string from the URL, matching the model's string primary key. Typed parameters like `{note_id:int}` also exist — see the [Routing guide](../guides/routing).
- `req.user_id` and `req.db` were set by middleware — see [Requests](../guides/requests) for dynamic attributes.

## 8. Wiring it together

`app.py` — the router layers middleware sequentially: public routes first (signup/signin), then JWT-protected routes:

```python
import asyncio

from oxapy import Oxapy, Router

from .config import HOST, PORT
from .db import init_db
from .middlewares import auth, db
from .routes import (
    create_note,
    delete_note,
    me,
    note_detail,
    notes,
    signin,
    signup,
)


async def run_server():
    init_db()
    await (
        Oxapy((HOST, PORT))
        .max_connections(1000)
        .attach(
            Router("/api")
            .middleware(db)
            .routes([signup, signin])
            .middleware(auth)
            .routes([me, notes, create_note, note_detail, delete_note])
        )
        .async_mode()
        .run()
    )


if __name__ == "__main__":
    asyncio.run(run_server())
```

Key points:

- `Router("/api")` prefixes every route, so `/signup` becomes `/api/signup`.
- Middleware is layered: everything gets `db`; only the second group gets `auth`. Routes registered *after* a middleware are covered by it — see [Middleware](../guides/middleware).
- `async_mode()` enables `async def` handlers; `run()` returns an awaitable.

## 9. Run it

```bash
pip install oxapy sqlalchemy
python -m notes_api.app
```

```bash
# Sign up
curl -X POST http://127.0.0.1:8000/api/signup \
  -H "Content-Type: application/json" \
  -d '{"email": "ada@example.com", "full_name": "Ada Lovelace", "password": "supersecret"}'

# {"user": {"id": "...", "email": "ada@example.com", "full_name": "Ada Lovelace", "created_at": "..."}}

# Log in and save the token
curl -X POST http://127.0.0.1:8000/api/signin \
  -H "Content-Type: application/json" \
  -d '{"email": "ada@example.com", "password": "supersecret"}'
# {"user": {...}, "token": "eyJhbGciOiJIUzI1NiIs..."}

# Create a note (authenticated)
curl -X POST http://127.0.0.1:8000/api/notes \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"title": "First note", "content": "Hello OxAPY"}'

# List notes
curl http://127.0.0.1:8000/api/notes -H "Authorization: Bearer <token>"

# Read one note (typed path parameter)
curl http://127.0.0.1:8000/api/notes/<note_id> -H "Authorization: Bearer <token>"

# Delete a note
curl -X DELETE http://127.0.0.1:8000/api/notes/<note_id> -H "Authorization: Bearer <token>"
```

Without a token you get `401`; with a wrong note id you get a JSON `404`:

```json
{"detail": "Note not found"}
```

## What you just learned

| Feature | Where it is used here | Learn more |
| --- | --- | --- |
| Route decorators + path params | `@get("/notes/{note_id}")` | [Routing](../guides/routing) |
| Router base path + middleware layering | `Router("/api")`, `.middleware(db).routes([...])` | [Routing](../guides/routing), [Middleware](../guides/middleware) |
| Request reading | `req.json()`, `req.data`, `req.headers` | [Requests](../guides/requests) |
| Response conversion | `(dict, Status.CREATED)` tuples, `Status.OK` | [Responses](../guides/responses) |
| Serializers with `Meta.model` | `UserSerializer`, `NoteSerializer` | [Serializers](../guides/serializers) |
| JWT auth | `Jwt.generate_token`, `verify_token` | [JWT Authentication](../guides/jwt-authentication) |
| Exceptions → JSON errors | `exceptions.NotFoundError` | [Error Handling](../guides/error-handling) |
| Async mode | `async def` handlers, `await ...run()` | [Async Handlers](../guides/async-handlers) |
| Server options | `max_connections(1000)` | [Server Configuration](../advanced/server-configuration) |

## Taking it further

- Add [templates](../guides/templates) and [sessions](../guides/sessions) to render a browser UI (with HTMX, like the job board pattern).
- Serve assets with [static_file](../guides/static-files) and large uploads with [file-streaming](../guides/file-streaming).
- Enable [hot reload](../guides/hot-reload) during development with `Oxapy(...).run(reload=True)`.
- Read the [API Reference](../api/server) for every option on `HttpServer` and `Oxapy`.
