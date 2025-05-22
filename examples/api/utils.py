from oxapy import jwt
import bcrypt

SECRET = "8b78e057cf6bc3e646097e5c0277f5ccaa2d8ac3b6d4a4d8c73c7f6af02f0ccd"

jwt = jwt.Jwt(SECRET)


def create_jwt(user_id: str) -> str:
    payload = {"user_id": user_id}
    return jwt.generate_token(payload)


def decode_jwt(token: str):
    return jwt.verify_token(token)


def hash_password(password: str) -> str:
    return bcrypt.hashpw(password.encode(), bcrypt.gensalt()).decode()


def check_password(hashed_password: str, password: str) -> bool:
    return bcrypt.checkpw(password.encode(), hashed_password.encode())
