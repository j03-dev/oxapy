from oxapy import Oxapy, Router, get


@get("/hello/{name}")
async def hello(_req, name):
    return f"Hello, {name}!"


async def main():
    await Oxapy(("127.0.0.1", 5555)).attach(Router().route(hello)).async_mode().run()


if __name__ == "__main__":
    import asyncio

    asyncio.run(main())
