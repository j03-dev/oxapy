from oxapy import Oxapy, Router, get, post


def main():
    (
        Oxapy(("0.0.0.0", 3000))
        .channel_capacity(1000 * 8)
        .max_connections(1000 * 8)
        .attach(
            Router()
            .route(get("/", lambda _: ""))
            .route(get("/user/{id:int}", lambda _, id: str(id)))
            .route(post("/user", lambda _: ""))
        )
        .run(workers=1, processes=8)
    )


if __name__ == "__main__":
    main()
