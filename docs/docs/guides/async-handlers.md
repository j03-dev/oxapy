# Async Handlers

Handlers in async mode must be `async def` coroutines. Enable async mode with `async_mode()` and await the server.

## A minimal async app

```python
import asyncio

from oxapy import Oxapy, Router, get


@get("/")
async def home(request):
    # Asynchronous operations are allowed here
    data = await fetch_data_from_database()
    return "Hello, World!"


async def main():
    await (
        Oxapy(("127.0.0.1", 8000))
        .attach(Router().route(home))
        .async_mode()
        .run()
    )


if __name__ == "__main__":
    asyncio.run(main())
```

The two differences from a sync app:

1. Handlers are declared with `async def` and use `await`.
2. The server is started with `app.async_mode().run()` inside an async `main()`, which you run with `asyncio.run(main())`.

## Handlers in async mode cannot be synchronous

`async_mode()` is a server-wide setting. When async mode is enabled, **all** route handlers registered on the server must be `async def` coroutines.

You cannot mix synchronous handlers and `async def` handlers on the same server instance. The async runtime expects every handler execution to return an awaitable/coroutine; if a handler returns a plain synchronous value (such as a `str` or `dict`), the runtime fails when attempting to convert it into a future and returns a `500 Internal Server Error`.

```python
# ❌ INCORRECT in async mode: plain def returns a string, causing an exception
@get("/sync")
def sync_handler(request):
    return "I will cause an exception in async mode"


# ✅ CORRECT in async mode: declare with async def
@get("/async")
async def async_handler(request):
    result = await some_io()
    return {"result": result}
```

If you need to perform blocking synchronous I/O or CPU-bound work inside an async app, run it inside a worker thread using `asyncio.to_thread`:

```python
import asyncio


@get("/compute")
async def compute(request):
    result = await asyncio.to_thread(blocking_call)
    return {"result": result}
```

## When to use async mode

Use `async_mode()` when handlers perform non-blocking I/O — HTTP calls, database queries through an async driver, websocket clients. It keeps the event loop free while those operations are in flight.

For blocking I/O inside ordinary sync handlers, prefer `.handler_threads(n)` on the server instead: it runs up to `n` sync handlers on separate OS threads. The setting is ignored in async mode, and neither option bypasses the GIL for CPU-bound Python.

:::note

In async mode, `app.run()` returns an awaitable; calling it from a synchronous `main()` without awaiting will not start the server. Use the pattern above with `asyncio.run()`.

:::

## Next steps

- [Server Configuration](../advanced/server-configuration) — `async_mode()` and other server options
- [API Reference: Server](../api/server) — the `async_mode()` method
