import sys
import time

from oxapy import Oxapy, Router, get


@get("/health")
def health(_request):
    return {"message": "ok"}


@get("/slow")
def slow(_request):
    time.sleep(0.25)
    return {"message": "slow"}


def main(port, threads):
    app = Oxapy(("127.0.0.1", port)).attach(Router().routes([health, slow]))
    app.handler_threads(threads)
    app.run()


if __name__ == "__main__":
    main(int(sys.argv[1]), int(sys.argv[2]))
