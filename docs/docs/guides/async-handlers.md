# Async Handlers

Handlers can be plain functions or `async def` coroutines. Enable async mode with `async_mode()` and await the server.

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

## Mixing sync and async handlers

You do not have to choose one style. Register both plain and async handlers on the same router; each is called appropriately:

```python
@get("/sync")
def sync_handler(request):
    return "I run synchronously"


@get("/async")
async def async_handler(request):
    result = await some_io()
    return {"result": result}
```

## When to use async mode

Use `async_mode()` when handlers perform non-blocking I/O — HTTP calls, database queries through an async driver, websocket clients. It keeps the event loop free while those operations are in flight.

:::note

In async mode, `app.run()` returns an awaitable; calling it from a synchronous `main()` without awaiting will not start the server. Use the pattern above with `asyncio.run()`.

:::

## Next steps

- [Server Configuration](../advanced/server-configuration) — `async_mode()` and other server options
- [API Reference: Server](../api/server) — the `async_mode()` method
