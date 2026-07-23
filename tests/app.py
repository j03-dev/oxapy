import asyncio

from oxapy import Oxapy, Router, get


@get("/greet/{name}")
def greet(_r, name: str):
    return f"Hello, {name}!"


async def main():
    oxapy = Oxapy(("0.0.0.0", 5555))
    oxapy.set_patterns(["*.py", "*.html"])
    oxapy.attach(Router().route(greet).route(get("/health", lambda _: "Good")))
    oxapy.async_mode()
    await oxapy.run(reload=True)


if __name__ == "__main__":
    asyncio.run(main())
