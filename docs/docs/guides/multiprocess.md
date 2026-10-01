# Multiprocess Serving

`Oxapy` can serve your app from several **OS processes** at once. Each process runs the whole server — its own routers, middleware, templates, and Tokio runtime — and they all accept connections on the same port.

```python
from oxapy import Oxapy, Router, get


@get("/")
def home(request):
    return {"status": "ok"}


def main():
    (
        Oxapy(("127.0.0.1", 5555))
        .attach(Router().route(home))
        .run(processes=4)
    )


if __name__ == "__main__":
    main()
```

## `workers` vs `processes`

These two options control different levels of concurrency and are easy to confuse:

| Option | Level | Default | Notes |
| --- | --- | --- | --- |
| `workers` | Tokio worker **threads** inside each process | runtime decides (usually one per CPU core) | Ignored in async mode — see [Limitations](#limitations) |
| `processes` | **OS processes** | `1` (no supervisor) | Unix only; uses `SO_REUSEPORT` |

They compose. `run(processes=4, workers=2)` gives you 4 processes with 2 Tokio threads each — 8 threads of execution for the same port.

Because the GIL serializes Python bytecode inside a process, `processes` is usually the more effective knob for CPU-bound handlers, while `workers` helps I/O-bound ones.

## How it works

The `processes` argument is handled by a **supervisor written in Python** (`Oxapy._run_supervisor`), not by the Rust core:

1. `run()` checks the `OXAPY_WORKER` environment variable. If it is set to `1`, this process is already a child, so it calls straight through to the Rust server and stops there. This is the recursion guard.
2. Otherwise, the supervisor spawns `processes` children with `subprocess.Popen([sys.executable] + sys.argv, env=env)`, setting `OXAPY_WORKER=1` in their environment.
3. Each child binds the same address. This works because the listener socket sets `SO_REUSEPORT` (Unix only), so the kernel load-balances incoming connections across the processes.

:::warning The `if __name__ == "__main__":` guard is mandatory

Children are spawned by **re-executing your script with the same arguments** (`[sys.executable] + sys.argv`). If your module builds and runs a server at import time without the guard, every child will spawn its own children and you will get a fork bomb. The guard is what stops the recursion:

```python
if __name__ == "__main__":
    main()
```

Because `sys.argv` is replayed verbatim, any CLI flags your entry point accepts are also re-applied in each child. Keep them idempotent.

:::

## Restart behavior

Without `reload=True`, the pool is still self-healing:

- A worker that **crashes** (non-zero exit) is respawned on its own, and the others keep serving. The supervisor prints `Worker {i} exited with code {n}, restarting...`.
- A worker that **exits cleanly** (exit code `0`) is left down. Once every worker has exited cleanly, the supervisor itself returns.
- With `reload=True`, a crash restarts the **entire pool** instead — the surviving workers would otherwise still be serving pre-crash code. A clean exit is still left down, so the pool shrinks until the next reload. See the [Hot Reload guide](./hot-reload).

## Shutdown

`Ctrl-C` (SIGINT) and `SIGTERM` both unwind the supervisor. SIGTERM is translated into `KeyboardInterrupt` so both paths run identical cleanup (`Oxapy._run_supervisor` installs a temporary SIGTERM handler, then restores the previous one on exit).

Workers are stopped with SIGTERM, given **3 seconds** to exit, and then killed with SIGKILL.

:::note Shutdown is not graceful

There is no request drain. When the supervisor tears the pool down — on `Ctrl-C`, on `SIGTERM`, or on a reload — in-flight requests are cut off rather than being allowed to finish. See [Limitations](#limitations).

:::

## Limitations

These are real constraints of the current implementation. Read them before you put `processes > 1` into production.

### Unix only

`SO_REUSEPORT` is only set on Unix platforms. On Windows only the **first** process binds the port; the remaining children fail to bind, exit non-zero, and get respawned by the supervisor — a silent crash loop that will fill your logs with bind errors.

Windows can still run multiple processes, but not through this mechanism: run several independent server instances on separate ports behind a load balancer or reverse proxy instead.

### No graceful connection drain

The request pump stops as soon as the shutdown signal arrives; it does not wait for in-flight connections to complete (`src/lib.rs`, the `process_requests` loop). Expect dropped connections during restarts and deploys.

### `workers` is ignored in async mode

In async mode, `run()` returns an awaitable and the `workers` argument is **not** forwarded to the Tokio runtime. Passing `workers=` alongside `.async_mode()` has no effect.

### In-memory state is not shared

Each process holds its own copy of everything you attach to the server, including `request.app_data` and any module-level caches. Anything that must be consistent across processes needs a shared store (database, Redis). See [Application State](./app-state).

Session and CSRF cookies are unaffected — they are signed values, so all processes validate them identically with the same secret.

### Not a full process manager

This is a convenience pool, not a Gunicorn replacement. There is no pre-fork socket handoff, no `max_requests` worker recycling to contain memory leaks, no PID file, no readiness probe, and no zero-downtime rolling restart.

## When to use it

- You need more than one core for CPU-bound Python handlers.
- You want a self-healing pool without introducing an external dependency.

Prefer a single process with tuned `workers` when your handlers are I/O-bound or already use async — the supervisor adds a process layer for little gain. For long-running production deployments, [Deployment](../advanced/deployment) covers running behind a reverse proxy and a service manager.

## Next steps

- [Hot Reload](./hot-reload) — restart the pool automatically on file changes
- [Application State](./app-state) — what to do about state that must be shared
- [Deployment](../advanced/deployment) — running OxAPY in production
- [API Reference: Server](../api/server) — the `run` signature and defaults