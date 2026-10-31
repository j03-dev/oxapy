import asyncio
import sys

from oxapy import HttpServer, Router, get


@get("/health")
async def health(_request):
    return {"message": "ok"}


@get("/slow")
async def slow(_request):
    await asyncio.sleep(0.5)
    return {"message": "slow"}


async def main(port):
    app = HttpServer(("127.0.0.1", port)).attach(Router().routes([health, slow]))
    await app.async_mode().run()


if __name__ == "__main__":
    asyncio.run(main(int(sys.argv[1])))
