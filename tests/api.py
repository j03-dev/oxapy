from oxapy import HttpServer, Router, Request, Response
import asyncio


router = Router()
@router.get("/greet/{name}")
async def hi(request: Request, name: str):
    return f"Hello, {name}!"


server = HttpServer(("0.0.0.0", 5555))
server.attach(router)


async def main():
    await server.async_mode().run()

if __name__ == "__main__":
    asyncio.run(main())
