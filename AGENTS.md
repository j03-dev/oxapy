# AGENTS.md - Agentic Coding Guidelines for OxAPY

## Project Overview

OxAPY is a Python HTTP server library built in Rust using PyO3/maturin. It provides a fast, feature-rich web framework with routing, middleware, sessions, JWT authentication, and static file serving.

## Build, Lint, and Test Commands

### Building the Project

```bash
# Development build (installs in editable mode)
uv run maturin dev --release

# Or use the build script
./build.sh

# Build wheel for distribution
uv run maturin build --release
```

### Running E2E Tests

```bash
# Run all tests
pytest -vv tests

# Run a single test file
pytest -vv tests/test_http_server.py

# Run a single test
pytest -vv tests/test_http_server.py::test_ping_endpoint

# Run with specific test markers
pytest -vv tests -k "test_name_pattern"
```

### Rust Linting and Formatting

```bash
# Format Rust code
cargo fmt

# Run clippy lints
cargo clippy --all-targets --all-features

# Check Rust code
cargo check --all-targets
```

### Pre-commit Hooks

The project uses pre-commit hooks defined in `.pre-commit-config.yaml`:
- Rust: `cargo fmt` and `cargo clippy`
- Python: `maturin develop --release` and `pytest -vv tests`

```bash
# Install pre-commit hooks
pre-commit install

# Run all hooks manually
pre-commit run --all-files
```

### Documentation Site (Docusaurus)

The project ships a Docusaurus 3 site in `docs/` that teaches the library through guides, a full tutorial, and an API reference. Run all docs commands from `docs/`:

```bash
# Validate + build the site (required safety net: broken links fail the build)
cd docs && npx docusaurus build

# Preview locally (dev server runs indefinitely - always bound with timeout)
timeout 180 npx docusaurus start --no-open

# Serve the built site from docs/build
cd docs && npm run serve
```

Notes:
- Prefer `npx docusaurus build` over `npm run build` (npm wrapper has intermittently failed with "authorization channel closed").
- The site is deployed to GitHub Pages at `https://j03-dev.github.io/oxapy/` (`baseUrl: '/oxapy/'` in `docusaurus.config.ts`). Docusaurus 3.x rejects a sub-path inside `url` when `baseUrl` is `/` — keep them as-is.
- Doc content must match the real API: verify facts against `oxapy/oxapy/*.pyi` stubs, `src/*.rs`, and `tests/` before writing.
- Search is client-side via `@easyops-cn/docusaurus-search-local` (no Algolia account needed); configured in the `plugins` array of `docusaurus.config.ts`. The index (`search-index.json`) is generated at build time.

### Deploying to GitHub Pages

The site is served from `https://j03-dev.github.io/oxapy/`. Two ways to publish:

1. **Manual (CLI)** — build and push to the `gh-pages` branch:
   ```bash
   cd docs && GIT_USER=j03-dev npx docusaurus deploy
   ```
   Requires the `gh-pages` branch to already exist on the remote — Docusaurus 3.10.2 cannot bootstrap a missing one (it fails with `Remote branch gh-pages not found in upstream origin`). Create it once if needed:
   ```bash
   git checkout --orphan gh-pages && git commit --allow-empty -m "init gh-pages" && git push origin gh-pages && git checkout -
   ```
   GitHub Pages setting: **Source = "Deploy from a branch"**, branch `gh-pages`, folder `/(root)`.

2. **Automatic (CI)** — `.github/workflows/deploy-docs.yml` builds and deploys via GitHub Actions on every push to `main`. Requires the GitHub Pages setting **Source = "GitHub Actions"** instead.

#### Writing / Editing Docs

- Markdown pages live in `docs/docs/<category>/<page>.md`; register every new page in `docs/sidebars.ts`.
- Use relative links, and mind the depth prefix: from `tutorial/`, `guides/`, `advanced/`, or `api/` pages, links to a sibling category need `../` (e.g. `../guides/routing`); from `intro.md` use `./guides/routing`. The `onBrokenLinks: 'throw'` build is the safety net — never ship a doc change without running `npx docusaurus build`.
- API pages (`docs/docs/api/*`) document the Python surface; guides (`docs/docs/guides/*`) teach usage with examples; `docs/docs/tutorial/notes-api.md` walks through a complete production-style app (SQLAlchemy + serializers + JWT + async mode).

## Project Structure

```
oxapy/
├── Cargo.toml                 # Rust crate config (pyo3 cdylib + rlib)
├── pyproject.toml             # Python packaging (maturin)
├── build.sh                   # Build helper script
├── src/
│   ├── lib.rs                 # HttpServer, server loop, request dispatch, PyModule registration
│   ├── middleware.rs           # Middleware chain builder (sequence-based wrapping)
│   ├── request.rs             # Request struct, RequestBuilder, cookie/JSON/form parsing
│   ├── response.rs            # Response struct, Redirect, FileStreaming, header manipulation
│   ├── routing.rs             # Route, Router, HTTP method decorators (get/post/etc), matchit
│   ├── status.rs              # Status enum (all HTTP status codes)
│   ├── into_response.rs       # convert_to_response: normalizes handler returns to Response
│   ├── exceptions.rs          # BadRequest/Unauthorized/Forbidden/NotFound/Conflict/InternalError
│   ├── cors.rs                # CORS config and header injection
│   ├── jwt.rs                 # JWT encode/decode (jsonwebtoken crate)
│   ├── json.rs                # JSON serialization (wraps orjson)
│   ├── multipart.rs           # Multipart form/file parsing (multer crate)
│   ├── templating.rs          # Tera template engine + render() function
│   └── serializer/
│       ├── mod.rs             # Serializer class (DRF-style: validate, create, save, update)
│       └── fields.rs          # Field types (Char, Email, Integer, Boolean, Number, UUID, etc.)
├── oxapy/
│   ├── __init__.py            # Python re-exports + Oxapy (hot-reload), Session, CsrfProtect, static_file
│   ├── __init__.pyi           # Auto-generated type stubs
│   ├── jwt/__init__.pyi       # JWT stubs
│   ├── exceptions/__init__.pyi # Exception stubs
│   ├── serializer/__init__.pyi # Serializer stubs
│   └── templating/__init__.pyi # Template stubs
└── tests/
    ├── conftest.py            # Test server fixture (Oxapy on port 9999, auth middleware demo)
    ├── app.py                 # Minimal async Oxapy example
    ├── test_http_server.py    # Integration tests (ping, echo, forms, uploads, auth, redirects)
    ├── test_session.py        # JWT encode/decode tests
    ├── test_response.py       # Response/Redirect unit tests
    ├── test_cors.py           # CORS config tests
    ├── test_serializer.py     # Serializer tests
    ├── test_exceptions.py     # Exception tests
    └── utils.py               # Multipart test helper
```

## Architecture

### Request Lifecycle

```
Client → TcpListener → RequestBuilder → Request::process()
  → OPTIONS + CORS configured? → return preflight response
  → iterate routers → router.find(method, uri) via matchit
  → if match: create ProcessRequest → mpsc channel
  → process_requests loop:
      → build middleware chain (sequence-based wrapping)
      → call_python_handler: middleware chain → Python handler
      → convert_to_response (normalize return type)
      → wrapper(request, response)  [if HttpServer.wrap() configured]
      → response.apply_cors(headers)
      → send via oneshot channel → hyper Response → Client
```

### Middleware System

Middleware is **sequence-based** and wraps handlers. Each middleware receives a `next` keyword argument:

```python
def my_middleware(request, next, **kwargs):
    # runs BEFORE handler
    result = next(request, **kwargs)  # calls next layer (or handler)
    # result is the handler's return value
    return result
```

- `Router.middleware(fn)` registers middleware; it applies to routes registered **after** it
- Chain is built recursively: last middleware wraps the handler, second-to-last wraps that, etc.
- Multiple `Router` instances with different middleware can be attached to the same server

### Template System (Tera)

- Templates are loaded via `Template.load(glob_pattern)` (Tera syntax)
- Custom functions must be registered **before** `load()`: `template.register_function("name", fn)`
- `render(request, "template.html", context)` auto-injects:
  - `session` dict (if `Session` middleware is active and `request.session` exists)
  - `csrf_token` string (if `CsrfProtect` middleware is active and `request.csrf_token` exists)
  - `csrf_token` string (if `CsrfProtect` middleware is active and `request.csrf_token` exists)

### Python Modules (oxapy/__init__.py)

Pure Python features implemented in `oxapy/__init__.py`:
- **`Session(secret, max_age)`** — Signed cookie middleware (HMAC-SHA256)
- **`CsrfProtect(secret, ...)`** — CSRF protection middleware + `csrf_input()` template helper
- **`secure_join(base, *paths)`** — Path traversal protection
- **`static_file(path, directory)`** — Static file serving route
- **`send_file(path)`** — File response helper

Rust-implemented features exposed via PyO3:
- `HttpServer`, `Router`, `Route`, `Request`, `Response`, `Status`, `Cors`, `Redirect`, `FileStreaming`
- HTTP method decorators: `get`, `post`, `put`, `patch`, `delete`, `head`, `options`
- `templating.Template`, `templating.render`
- `jwt.Jwt` (encode/decode), `exceptions.*`, `serializer.Serializer`

## Code Style Guidelines

### General Project Structure

- **Rust source**: `src/` directory with modular `.rs` files
- **Python source**: `oxapy/__init__.py` for pure-Python features, Rust for core server
- **Python tests**: `tests/` directory
- **Docs site**: `docs/` directory (Docusaurus; markdown in `docs/docs/`)

### Rust Code Conventions

1. **Imports**: Use absolute imports from crate root
   ```rust
   use crate::routing::*;
   use crate::middleware::Middleware;
   ```

2. **PyO3 Patterns**:
   - Use `#[pyclass]` for Python-exposed structs
   - Use `#[pymethods]` for methods callable from Python
   - Use `#[gen_stub_pyclass]` and `#[gen_stub_pymethods]` for stub generation
   - Use `#[new]` for constructors
   - Use `#[pyo3(signature=(...))]` for keyword arguments

3. **Naming**:
   - Structs/Enums: `PascalCase`
   - Functions/Methods: `snake_case`
   - Constants: `SCREAMING_SNAKE_CASE`

4. **Error Handling**:
   - Return `PyResult<T>` for functions that can raise Python exceptions
   - Use the `IntoPyException` trait for custom error types
   - Use `pyo3::exceptions::*` for standard Python exceptions

5. **Documentation**:
   - Use doc comments `///` for public APIs
   - Include Args, Returns, and Example sections in docstrings

6. **Concurrency**:
   - Use `Arc<T>` for shared ownership
   - Use `tokio` for async runtime with `pyo3-async-runtimes`

### Python Code Conventions

1. **Imports**: Follow standard Python import conventions
   ```python
   from oxapy import HttpServer, Router, get, post, Status, Response
   ```

2. **Type Hints**: Use type hints for function signatures
   ```python
   @get("/hello/{name}")
   def hello(_request, name: str) -> dict:
       return {"message": f"Hello, {name}!"}
   ```

3. **Handler Functions**:
   - First argument is always `request`
   - Path parameters are passed as keyword arguments
   - Return type can be `str`, `dict`, `Response`, or `Status`

### Key Dependencies

- **pyo3**: Python bindings (>=0.27.0)
- **pyo3-async-runtimes**: Async support with tokio-runtime
- **tokio**: Async runtime
- **hyper**: HTTP server
- **matchit**: URL routing
- **serde/serde_json**: Serialization
- **minijinja/tera**: Template engines
- **jsonwebtoken**: JWT authentication

### Testing Patterns

Tests use a session-scoped fixture that starts a real HTTP server:

```python
@pytest.fixture(scope="session")
def oxapy_server(static_files_dir):
    thread = threading.Thread(target=lambda: main(static_files_dir), daemon=True)
    thread.start()
    time.sleep(2)  # Wait for server to start
    yield "http://127.0.0.1:9999"
```

Use `requests` library for HTTP assertions in tests.

### Common Patterns

1. **Creating a Router**:
   ```python
   # Create router and register decorated handlers with .routes()
   router = Router("/api/v1")
   router.routes([handler1, handler2])
   
   # Or use .route() for single handler
   router.route(handler)
   ```

2. **Middleware**:
   ```python
   def auth_middleware(request, next, **kw):
       if "authorization" not in request.headers:
           return Status.UNAUTHORIZED
       return next(request, **kw)
   ```

3. **Response Types**:
   - Return `str` for plain text
   - Return `dict` for JSON (auto-serialized)
   - Return `Response` object for custom responses
   - Return `Status` for error codes

### Important Notes

- The project uses `ahash` instead of standard `HashMap` for performance
- Path parameters in routes use `{param_name}` syntax
- Middleware applies to all routes registered **after** it within the same router; use separate `Router` instances to isolate middleware groups
- Multiple routers can be attached to the server and are checked in order until a matching route is found
- Application state is shared via `request.app_data`
