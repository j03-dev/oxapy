from models import Base
from sqlalchemy import create_engine


class AppData:
    def __init__(self):
        self.engine = create_engine("sqlite:///database.db")
        self.n = 0
        Base.metadata.create_all(self.engine)


APP_DATA = AppData()
ORIGINS = ["http://localhost:5555"]
METHODS = ["GET", "POST"]
