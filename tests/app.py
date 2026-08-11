from oxapy import Oxapy, Router, get


@get("/hello/{name}")
def hello(_req, name):
    return f"Hello, {name}!"


def main():
    Oxapy(("127.0.0.1", 5555)).attach(Router().route(hello)).run()


if __name__ == "__main__":
    main()
