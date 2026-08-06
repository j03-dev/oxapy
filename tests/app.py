from oxapy import Oxapy, Router, get


def main():
    (
        Oxapy(("127.0.0.1", 5555))
        .attach(
            Router().route(get("/greet/{name}", lambda _, name: f"Hello, {name}!"))
        )
        .run()
    )


if __name__ == "__main__":
    main()
