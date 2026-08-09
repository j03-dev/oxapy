# Templating

Template rendering for HTML pages, built on Tera.

## Template

The top-level `Template` class configures the engine.

### Constructor

```python
Template()
```

Creates an empty engine. Nothing is loaded until `load()` is called.

### Methods

#### register_function

```python
register_function(name: str, callable) -> None
```

Exposes a Python callable to templates. Must be called **before** `load()`; calling it afterwards raises `RuntimeError`.

```python
template = templating.Template()
template.register_function("add", lambda a, b: a + b)
```

In a template: `{{ add(a=1, b=2) }}` renders `3`.

#### load

```python
load(dir: str = "./templates/**/*.html") -> None
```

Parses and validates all templates matching the glob pattern. Raises `RuntimeError` if the engine is shared across references, and `PyException` for invalid globs or template errors.

### Attaching to the server

```python
server.template(template)
```

## render

The free function renders a template into a `Response`:

```python
render(request: Request, name: str, context: dict | None = None) -> Response
```

- Uses the template engine attached to the server
- Serves the result with `Content-Type: text/html`
- Makes `session` available to templates as `{{ session }}` when the session middleware set it
- Raises `ValueError` if no template engine is configured

```python
from oxapy import Router, get, render


@get("/")
def index(request):
    return render(request, "index.html", {"title": "Home Page"})
```

## Example: complete setup

```python
from oxapy import HttpServer, Router, get, render, templating


def translate(key):
    return translations.get(key, key)


def main():
    template = templating.Template()
    template.register_function("_t", translate)
    template.load("./templates/**/*.html")

    @get("/")
    def index(request):
        return render(request, "index.html", {"title": "Home"})

    (
        HttpServer(("127.0.0.1", 8000))
        .template(template)
        .attach(Router().route(index))
        .run()
    )
```

## Related

- [Templates guide](../guides/templates) — full walkthrough
- [Server](./server) — the `template()` method
