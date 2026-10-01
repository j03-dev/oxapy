# Deployment

OxAPY is a Python package, so deployment follows the usual Python story: build a wheel, install it, and run your app behind a process manager or reverse proxy.

## Building a wheel

```bash
# From the repository, with a Rust toolchain available:
uv sync
uv run maturin build --release
```

The wheel lands in `target/wheels/`:

```bash
pip install target/wheels/oxapy-0.11.1-cp311-cp311-manylinux_2_17_x86_64.manylinux2014_x86_64.whl
```

For end users, `pip install oxapy` fetches the prebuilt wheel from PyPI — no Rust toolchain needed.

## The production entry point

Keep the server code in a module so it can be imported (rather than only run as a script):

```python
# app.py
from oxapy import Oxapy, Router, get


@get("/")
def home(request):
    return {"service": "oxapy", "status": "ok"}


def create_app():
    return Oxapy(("127.0.0.1", 5555)).attach(Router().route(home))


if __name__ == "__main__":
    create_app().run(processes=4)
```

`processes=4` serves the app from four OS processes sharing the port. Keep the `if __name__ == "__main__":` guard — the worker pool is created by re-executing this script. `SO_REUSEPORT` is Unix only, so on Windows run separate instances on separate ports behind a load balancer instead. See the [Multiprocess guide](../guides/multiprocess).

Alternatively, stay in a single process and give the Tokio runtime more threads:

```python
if __name__ == "__main__":
    create_app().run(workers=4)
```

## Running with a process manager

A process manager keeps the server alive and restarts it if it crashes. With systemd, define a unit:

```ini
[Unit]
Description=OxAPY app
After=network.target

[Service]
WorkingDirectory=/srv/myapp
Environment=SECRET_KEY=change-me
ExecStart=/srv/myapp/.venv/bin/python -m app
Restart=always

[Install]
WantedBy=multi-user.target
```

Then `systemctl enable --now myapp`.

## Behind a reverse proxy

OxAPY speaks HTTP directly; put it behind Nginx or Caddy for TLS, gzip, and load balancing.

### Nginx

```nginx
server {
    listen 443 ssl;
    server_name api.example.com;

    location / {
        proxy_pass http://127.0.0.1:5555;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Caddy

```text
api.example.com {
    reverse_proxy 127.0.0.1:5555
}
```

## Production checklist

- Use `Oxapy` with the default `reload=False`. Never run with `reload=True` in production.
- Run behind a reverse proxy that terminates TLS. Session cookies and JWT secrets travel over the wire otherwise.
- Set a strong `Session` secret / JWT secret via environment variables.
- Tune `workers` to match the machine's CPU count and `max_connections` to your expected load. Add `processes` only if handlers are CPU-bound — remember it is Unix only and does not drain in-flight requests on restart.
- Keep anything that must be consistent across workers in a shared store. Each process has its own copy of `app_data` and module state.
- Serve static assets with a dedicated server (Nginx, CDN) when traffic is high, or keep them behind `static_file()` for small apps. `static_file()` guards against path traversal with `secure_join()`, but uploads written with `File.save()` need their own filename sanitizing.
- Budget for abrupt shutdowns: restarting a worker pool cuts in-flight requests rather than draining them. Use a draining proxy if you need zero-downtime deploys.

## Documentation site

This site is built with Docusaurus:

```bash
cd docs
npm install
npx docusaurus build   # static site in docs/build
npx docusaurus deploy  # publish to GitHub Pages
```

The configuration targets GitHub Pages at `https://j03-dev.github.io/oxapy`. If you publish under a project path, set `baseUrl: '/oxapy/'` in `docs/docusaurus.config.ts`.

## Next steps

- [Multiprocess](../guides/multiprocess) — the `processes` argument and its limits
- [Server Configuration](./server-configuration) — the full configuration reference
- [API Reference: Server](../api/server) — signatures and defaults
