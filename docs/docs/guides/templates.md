# Templates

OxAPY ships a template engine based on [Tera](https://keats.github.io/tera/) (a Jinja2-like language) for rendering server-side HTML. See the [Tera documentation](https://keats.github.io/tera/) for the full template syntax reference.

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
  <head>
    <title>{{ title }}</title>
  </head>
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

## Automatic template variables

The `render()` function injects variables into the template context based on active middleware:

| Variable     | Source                   | Description                                     |
| ------------ | ------------------------ | ----------------------------------------------- |
| `session`    | `Session` middleware     | The session dictionary from `request.session`   |
| `csrf_token` | `CsrfProtect` middleware | The CSRF token string from `request.csrf_token` |

When `CsrfProtect` is active, the built-in `csrf_input` template function is also available:

```html
<form method="POST" action="/submit">
  {{ csrf_input(token=csrf_token) }}
  <input type="text" name="username" />
  <button type="submit">Submit</button>
</form>
```

This renders a hidden `<input>` with the token — no manual passing required. See the [CSRF Protection guide](./csrf-protection) for details.

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
