# Templates

OxAPY ships a template engine based on [Tera](https://keats.github.io/tera/) (a Jinja2-like language) for rendering server-side HTML.

## Enabling templates

Create a `Template` instance, configure it, and attach it to the server:

```python
from oxapy import Oxapy, Router, get, render, templating


def main():
    template = templating.Template()  # loads ./templates/**/*.html by default

    (
        Oxapy(("127.0.0.1", 5555))
        .template(template)
        .attach(Router().route(index))
        .run()
    )
```

The template directory defaults to `./templates/**/*.html`. Pass a glob pattern to use another directory:

```python
template = templating.Template("./views/**/*.html")
```

## Rendering from a handler

The `render(request, name, context)` function renders a template and returns an HTML response:

```python
from oxapy import Router, get, render


@get("/")
def index(request):
    return render(request, "index.html", {"title": "Home Page"})
```

Given this template file `templates/index.html`:

```html
<!DOCTYPE html>
<html>
  <head><title>{{ title }}</title></head>
  <body>
    <h1>{{ title }}</h1>
    <ul>
      {% for item in items %}
      <li>{{ item }}</li>
      {% endfor %}
    </ul>
  </body>
</html>
```

The rendered page is served with `Content-Type: text/html`. When the `session` is set on the request, it is made available to templates as `{{ session }}` automatically.

## Custom template functions

Register Python functions that templates can call, such as a translation helper:

```python
def translate(key):
    translations = {"hello": "Bonjour"}
    return translations.get(key, key)


def main():
    template = templating.Template()
    template.register_function("_t", translate)
    template.load()
    ...
```

```html
<p>{{ _t(key="hello") }}</p>
```

:::warning

Register all custom functions **before** calling `load()`. Tera validates function references when templates are loaded, and `register_function()` raises a `RuntimeError` after `load()` has been called.

:::

## Template lifecycle

- `Template()` creates an empty engine; nothing is loaded yet.
- `register_function(name, callable)` exposes a Python callable to templates (must be called before `load()`).
- `load(dir="./templates/**/*.html")` parses and validates all matching templates.
- `render(request, name, context)` renders a template and returns a `Response`.

## Next steps

- [API Reference: Templating](../api/templating) — the full template API
- [Responses](./responses) — building HTML responses by hand
