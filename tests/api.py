from oxapy import HttpServer, Router
import asyncio


router = Router()
@router.get("/greet/{name}")
async def hi(request, name):
    return f"Hello, {name}!"


server = HttpServer(("0.0.0.0", 5555))
server.attach(router)

async def main():
    await server.run(is_async=True)

if __name__ == "__main__":
    asyncio.run(main())
