# OxAPY TODO

Roadmap based on feature gap analysis against Flask, FastAPI, Django, Litestar, Starlette, Sanic, BlackSheep.

---

## Critical (P0)

### 1. WebSocket Support
- [ ] Add WebSocket upgrade handling via hyper's `hyper::upgrade::on()`
- [ ] Expose `WebSocket` class to Python with `send()`, `receive()`, `close()`
- [ ] Add `@ws("/path")` decorator or `Router.websocket()` method
- [ ] Support async handlers for WebSocket
- [ ] Tests: echo server, broadcast, concurrent connections

### 2. Dependency Injection
- [ ] Add `Depends()` function that registers a callable dependency
- [ ] Resolve dependency graph per-request (with caching per-request scope)
- [ ] Support nested dependencies (`Depends(get_db)` -> `Depends(get_user)`)
- [ ] Support overriding dependencies in tests (`app.override(Depends(get_db), mock_db)`)
- [ ] Integrate with OpenAPI schema generation (if implemented)
- [ ] Tests: nested deps, override in tests, caching

### 3. OpenAPI / Swagger Auto-Generation
- [ ] Generate OpenAPI 3.x schema from route definitions, serializers, and type hints
- [ ] Serve Swagger UI at `/docs` and ReDoc at `/redoc`
- [ ] Auto-document path params, query params, request body, response models
- [ ] Support `response_model` parameter on route decorators
- [ ] Tests: schema correctness, UI serving

### 4. Signature-Based Request Validation
- [ ] Inspect handler function signatures at registration time
- [ ] Auto-parse and validate query params, headers, cookies from type hints
- [ ] Auto-parse request body from Pydantic/dataclass/serializer models
- [ ] Return structured validation errors (422 Unprocessable Entity)
- [ ] Support `Annotated[type, Query()]`, `Annotated[type, Header()]`, etc.
- [ ] Tests: query validation, header extraction, body parsing, error responses

### 5. Background Tasks
- [ ] Add `BackgroundTasks` class with `add_task(func, *args, **kwargs)`
- [ ] Execute tasks after response is sent to client
- [ ] Support async background tasks
- [ ] Pass `BackgroundTasks` instance to handler via DI or parameter
- [ ] Tests: task execution after response, async tasks

### 6. Lifespan Events (Startup/Shutdown)
- [ ] Add `on_startup(func)` and `on_shutdown(func)` to `HttpServer`
- [ ] Support async startup/shutdown hooks
- [ ] Execute startup hooks before accepting connections
- [ ] Execute shutdown hooks on Ctrl+C (before stopping)
- [ ] Support `@app.on_startup` / `@app.on_shutdown` decorators
- [ ] Tests: hook execution order, async hooks

### 7. Generic Streaming Responses
- [ ] Add `StreamingResponse` class (generalize `FileStreaming`)
- [ ] Accept async generators, sync generators, or `StreamBody` as body source
- [ ] Support custom content type
- [ ] Use cases: SSE, NDJSON, LLM token streaming, real-time logs
- [ ] Tests: async generator streaming, chunked delivery

### 8. Test Client
- [ ] Add `TestClient(app)` that makes in-process HTTP requests
- [ ] No real TCP server needed (use hyper's in-process service)
- [ ] Support context manager (`with TestClient(app) as client:`)
- [ ] Support session/cookie persistence across requests
- [ ] Return response objects with `.status_code`, `.json()`, `.text`, `.headers`
- [ ] Tests: fast unit tests without port/thread conflicts

---

## Important (P1)

### 9. GZip Response Compression
- [ ] Add `GZipMiddleware` that compresses responses based on `Accept-Encoding`
- [ ] Configurable minimum response size threshold
- [ ] Support gzip and/or brotli
- [ ] Skip compression for streaming responses

### 10. Trusted Host / HTTPS Redirect
- [ ] Add `TrustedHostMiddleware` that validates `Host` header
- [ ] Add `HTTPSRedirectMiddleware` that redirects HTTP to HTTPS
- [ ] Configurable allowed hosts list

### 11. Client IP Address
- [ ] Expose `request.client.host` on the `Request` object
- [ ] Extract from `hyper`'s connected socket info
- [ ] Support `X-Forwarded-For` / `X-Real-IP` behind reverse proxy (configurable)

### 12. Response Cookies API
- [ ] Add `response.set_cookie(name, value, max_age, path, domain, httponly, secure, samesite)`
- [ ] Add `response.delete_cookie(name, path, domain)`
- [ ] Type-safe API instead of manual `insert_header("set-cookie", "...")`

### 13. CSRF Protection
- [ ] Add `CsrfMiddleware` that generates and validates CSRF tokens
- [ ] Support synchronizer token pattern (double submit cookie)
- [ ] Auto-exempt safe methods (GET, HEAD, OPTIONS)
- [ ] Configurable exempt routes/patterns
- [ ] Integrate with session middleware

### 14. Per-Status Error Handlers
- [ ] Add `@app.errorhandler(404)` decorator
- [ ] Add `@app.exception_handler(ExceptionType)` decorator
- [ ] Override default exception-to-status mapping
- [ ] Support custom error pages (HTML) and error responses (JSON)

### 15. URL Reverse Routing (`url_for`)
- [ ] Register route names alongside path patterns
- [ ] Add `url_for(route_name, **params)` function
- [ ] Generate correct URLs with path parameters filled in
- [ ] Useful for templates, redirects, and emails

### 16. OAuth2 / Security Utilities
- [ ] Add `OAuth2PasswordBearer(tokenUrl="/token")` dependency
- [ ] Add `HTTPBasic` dependency for HTTP Basic auth
- [ ] Add `APIKeyHeader` / `APIKeyQuery` dependencies
- [ ] Support OAuth2 scopes

### 17. Content Negotiation
- [ ] Inspect `Accept` header to determine response format
- [ ] Support multiple serializers per route (JSON, XML, MessagePack)
- [ ] Default to JSON, fallback based on client preference

---

## Nice-to-Have (P2)

### 18. Class-Based Views / Controllers
- [ ] Add `Controller` class that groups related route handlers
- [ ] Support shared middleware, pre/post hooks per controller
- [ ] Auto-register routes from controller methods

### 19. Blueprint / Module System
- [ ] Add `Blueprint` class for splitting routes across files
- [ ] Support `app.register_blueprint(bp, prefix="/api/v1")`
- [ ] Auto-merge middleware and static files from blueprints

### 20. CLI Runner
- [ ] Add `oxapy run app:main` command
- [ ] Auto-detect uvicorn-like reload in dev
- [ ] Support `--host`, `--port`, `--reload` flags

### 21. Rate Limiting
- [ ] Add `RateLimitMiddleware` with configurable limits
- [ ] Support per-IP and per-route limits
- [ ] Use in-memory store or pluggable backend (Redis)

### 22. Settings / Environment Configuration
- [ ] Add `Settings` base class (pydantic-settings style)
- [ ] Load from `.env` files and environment variables
- [ ] Type validation at startup

### 23. i18n / Localization
- [ ] Add `gettext`-style translation function
- [ ] Support locale detection from `Accept-Language` header
- [ ] Date/number formatting per locale

---

## Bug Fixes & Quality

### Stubs
- [ ] Fix `Session()` return type in `__init__.pyi` — should be `Callable`, not `Response`
- [ ] Remove `catcher` from `__init__.py.__all__` (doesn't exist)
- [ ] Remove `"from typing_extensions import Self"` from stub `__all__`
- [ ] Fix docstrings: `app_data()`, `attach()`, `wrap()` say `Returns: None` but return `self`

### Security
- [ ] Change `SameSite=Lax` to `SameSite=Strict` on session cookies (or make configurable)
- [ ] Add `Origin` / `Referer` header check in session middleware for state-changing methods
- [ ] Replace `unwrap()` in `insert_header` / `append_header` with proper error handling

### Performance
- [ ] Cache `Regex::new` in `parse_params_value` (slug parsing) — currently recompiles every call
- [ ] Evaluate middleware chain `py.eval()` overhead — consider alternatives

### Safety
- [ ] Replace `unsafe { std::mem::transmute }` in `request.rs:286` with safe alternative (e.g., `ouroboros` or restructure lifetime)

### Cleanup
- [ ] Remove `#![allow(unused_variables, non_snake_case)]` crate-level attribute — fix individually
- [ ] Add `CHANGELOG.md`
- [ ] Add `CONTRIBUTING.md`

---

## Testing

- [ ] Async mode tests (pytest-asyncio)
- [ ] Template rendering tests
- [ ] `FileStreaming` tests
- [ ] Session middleware E2E tests
- [ ] Serializer CRUD / schema / many=True tests
- [ ] JWT expiration + algorithm mismatch tests
- [ ] CORS integration tests (actual HTTP requests verifying headers)
- [ ] `wrap()` / error handler tests
- [ ] Typed path params (`:int`, `:slug`) tests
- [ ] `Request.get_cookie()` tests
- [ ] `Request` dynamic attributes tests
- [ ] `max_connections` limiting tests
- [ ] Hot reload tests
- [ ] Concurrent connections tests
