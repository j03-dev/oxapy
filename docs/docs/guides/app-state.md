# Application State

Application state is data shared across all requests and handlers. Attach any Python object to the server with `app_data()`, then read it from any handler via `request.app_data`.

## Setting app data

```python
from oxapy import Oxapy, Router, get


class AppState:
    def __init__(self):
        self.counter = 0
        self.db_pool = None  # e.g. a database connection pool


@get("/count")
def count(request):
    state = request.app_data
    state.counter += 1
    return {"count": state.counter}


def main():
    (
        Oxapy(("127.0.0.1", 5555))
        .app_data(AppState())
        .attach(Router().route(count))
        .run()
    )


if __name__ == "__main__":
    main()
```

Every request to `/count` increments the same counter:

```bash
curl http://127.0.0.1:5555/count   # {"count": 1}
curl http://127.0.0.1:5555/count   # {"count": 2}
```

## What to store

`app_data` accepts any Python object, which makes it the natural home for:

- Database connection pools and ORM sessions
- Caches and in-memory stores
- Configuration objects
- Shared services (mailers, queues, external API clients)

```python
class AppState:
    def __init__(self):
        self.counter = 0
        self.db_pool = create_database_pool()
```

## Notes

- If no `app_data` was set, `request.app_data` returns `None`.
- State lives in the server process. With multiple worker processes each worker has its own copy; use a shared store (database, Redis) for data that must be visible across workers.

## Next steps

- [Requests](./requests) — the full request API, including `app_data`
- [Server Configuration](../advanced/server-configuration) — other `HttpServer` options
- [API Reference: Server](../api/server) — the `app_data()` method
