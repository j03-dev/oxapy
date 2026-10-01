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

With `reload=True`, `Oxapy` acts as a **supervisor**: it spawns a worker process that runs the real server, and watches your files itself. When a watched file changes, the worker is restarted:

```
Reloading 1 worker(s)... (app.py changed)
```

:::warning The `if __name__ == "__main__":` guard is required

The worker is spawned by re-executing your script with `sys.argv`, so an unguarded entry point spawns workers that each spawn more workers. The example above includes the guard for this reason — keep it in any script that uses `reload=True`.

:::

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

1. `run(reload=True)` checks the `OXAPY_WORKER` environment variable. If it is not set, this process is the supervisor and starts the watching loop; otherwise it is already a worker and calls straight through to the Rust server. This is the recursion guard.
2. The supervisor starts a `watchdog` observer on the watch directory and spawns a worker subprocess running your script with `OXAPY_WORKER=1`.
3. When a watched file is created, modified, or deleted, the whole worker pool is terminated and a fresh pool is spawned.
4. If a worker **crashes**, the same pool restart happens — the surviving workers would otherwise keep serving pre-crash code.
5. A worker that exits **cleanly** (exit code `0`) is left down and is not respawned, so the pool shrinks until the next reload or a manual restart. The supervisor itself exits on `Ctrl-C` or `SIGTERM`.

## Reloading a pool of workers

Reload and multiprocess mode are the same supervisor, so `reload=True` combines with `processes=N` to reload every worker at once:

```python
Oxapy(("127.0.0.1", 5555)).attach(router).run(reload=True, processes=4)
```

The notice reflects the pool size:

```
Reloading 4 worker(s)... (app.py changed)
```

See the [Multiprocess guide](./multiprocess) for how the pool itself works.

## Notes

- This is a development feature. Keep `reload=False` (the default) in production.
- The watched script is re-run as a new process, so in-memory state does not survive a reload.
- Reloads are not graceful — in-flight requests are cut off when the pool is torn down.

## Next steps

- [Multiprocess](./multiprocess) — serving from several processes with `processes=N`
- [Deployment](../advanced/deployment) — running OxAPY in production
- [API Reference: Server](../api/server) — the `Oxapy` subclass methods
