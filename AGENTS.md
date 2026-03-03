# AGENTS.md - Agentic Coding Guidelines for OxAPY

## Project Overview

OxAPY is a Python HTTP server library built in Rust using PyO3/maturin. It provides a fast, feature-rich web framework with routing, middleware, sessions, JWT authentication, and static file serving.

## Build, Lint, and Test Commands

### Building the Project

```bash
# Development build (installs in editable mode)
maturin dev --release

# Or use the build script
./build.sh

# Build wheel for distribution
maturin build --release
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

## Code Style Guidelines

### General Project Structure

- **Rust source**: `src/` directory with modular `.rs` files
- **Python tests**: `tests/` directory

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
- Middleware applies to all routes in the same scope; use `.scope()` to create new middleware groups
- Application state is shared via `request.app_data`
