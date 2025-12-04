from oxapy import HttpServer, Router, get


def main():
    (
        HttpServer(("0.0.0.0", 5555))
        .max_connections(100)
        .channel_capacity(1000)
        .attach(
            Router()
            .route(get("/greet/{name}", lambda _r, name: f"Hello, {name}"))
        )
        .run(4)
    )


if __name__ == "__main__":
    main()
