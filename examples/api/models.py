from sqlalchemy.orm import Mapped, mapped_column, DeclarativeBase
from sqlalchemy import String


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
