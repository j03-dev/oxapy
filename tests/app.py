import multiprocessing
workers = multiprocessing.cpu_count()


from oxapy import Oxapy, Request, Router, get, post


@get("/hello/{name:str}")
def hello(r: Request):
    return "Hello, {name}"


 
def main():
    (
        Oxapy(("0.0.0.0", 3000))
        .attach(
            Router()
            .route(hello)
            .route(get("/", lambda _: ""))
            .route(get("/user/{id:int}", lambda _, id: str(id)))
            .route(post("/user", lambda _: ""))
        )
        .run(workers=workers)
    )


if __name__ == "__main__":
    main()
