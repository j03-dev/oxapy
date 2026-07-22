from oxapy import Oxapy, Router, get


@get("/greet/{name}")
def greet(_r, name: str):
    return f"Hello, {name}!"


def main():
    oxapy = Oxapy(("0.0.0.0", 5555))
    oxapy.set_patterns(["*.py", "*.html"])
    oxapy.attach(Router().route(greet).route(get("/health", lambda _: "Good")))
    oxapy.run(reload=True)


main()
