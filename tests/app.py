import multiprocessing

workers = multiprocessing.cpu_count()


from oxapy import Oxapy, Router, get, post


def main():
    (
        Oxapy(("0.0.0.0", 3000))
        .channel_capacity(1000 * 2)
        .max_connections(5000)
        .attach(
            Router()
            .route(get("/", lambda _: ""))
            .route(get("/user/{id:int}", lambda _, id: str(id)))
            .route(post("/user", lambda _: ""))
        )
        .run()
    )


if __name__ == "__main__":
    main()
