from oxapy import Oxapy, Router, get, post


@get("/")
def index(_):
    return ""


@get("/user/{id}")
def get_user(_, id: int):
    return str(id)


@post("/user")
def create_user(_):
    return ""


def main():
    (
        Oxapy(("0.0.0.0", 3000))
        .attach(Router().routes([index, get_user, create_user]))
        .run(workers=1, processes=8)
    )


if __name__ == "__main__":
    main()
