# Hot Reload

During development you want the server to restart automatically when you change code. The `Oxapy` class provides this with a file watcher (built on `watchdog`).

## Enabling reload

Use `Oxapy` (instead of `HttpServer`) and pass `reload=True` to `run()`:

```python
from oxapy import Oxapy, Router, get


@get("/")
def home(request):
    return "Hello, World!"


def main():
    (
        Oxapy(("127.0.0.1", 5555))
        .attach(Router().route(home))
        .run(reload=True)  # False by default
    )


if __name__ == "__main__":
    main()
```

With `reload=True`, `Oxapy` acts as a supervisor: it spawns a worker process that runs the real server and watches your files. When a watched file changes, the worker is restarted:

```
Reloading... (app.py changed)
```

## Watching specific patterns and directories

By default it watches `*.py` files under the current directory (`"."`). Adjust with `set_patterns()` and `set_watch_dir()`:

```python
(
    Oxapy(("127.0.0.1", 5555))
    .set_patterns(["*.py", "*.html"])   # also reload on template changes
    .set_watch_dir("src")               # watch only the src/ directory
    .attach(router)
    .run(reload=True)
)
```

Both methods return the instance, so they can be chained.

## How it works

1. `run(reload=True)` detects it is not a worker (the `OXAPY_WORKER` environment variable is not set) and starts the supervisor.
2. The supervisor starts a `watchdog` observer on the watch directory and spawns a worker subprocess running your script with `OXAPY_WORKER=1`.
3. When a watched file is created, modified, or deleted, the worker is terminated and a fresh one is spawned.
4. If the worker crashes, it is restarted automatically; the supervisor exits when the worker exits cleanly.

## Notes

- This is a development feature. Keep `reload=False` (the default) in production.
- The watched script is re-run as a new process, so in-memory state does not survive a reload.

## Next steps

- [Deployment](../advanced/deployment) — running OxAPY in production
- [API Reference: Server](../api/server) — the `Oxapy` subclass methods
