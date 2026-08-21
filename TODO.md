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

### 2. OpenAPI / Swagger Auto-Generation

- [ ] Generate OpenAPI 3.x schema from route definitions, serializers, and type hints
- [ ] Serve Swagger UI at `/docs` and ReDoc at `/redoc`
- [ ] Auto-document path params, query params, request body, response models
- [ ] Support `response_model` parameter on route decorators
- [ ] Tests: schema correctness, UI serving

### 3. Background Tasks

- [ ] Add `BackgroundTasks` class with `add_task(func, *args, **kwargs)`
- [ ] Execute tasks after response is sent to client
- [ ] Support async background tasks
- [ ] Pass `BackgroundTasks` instance to handler via DI or parameter
- [ ] Tests: task execution after response, async tasks

### 4. Generic Streaming Responses

- [ ] Add `StreamingResponse` class (generalize `FileStreaming`)
- [ ] Accept async generators, sync generators, or `StreamBody` as body source
- [ ] Support custom content type
- [ ] Use cases: SSE, NDJSON, LLM token streaming, real-time logs
- [ ] Tests: async generator streaming, chunked delivery

### 5. Test Client

- [ ] Add `TestClient(app)` that makes in-process HTTP requests
- [ ] No real TCP server needed (use hyper's in-process service)
- [ ] Support context manager (`with TestClient(app) as client:`)
- [ ] Support session/cookie persistence across requests
- [ ] Return response objects with `.status_code`, `.json()`, `.text`, `.headers`
- [ ] Tests: fast unit tests without port/thread conflicts

---

## Important (P1)

### 6. GZip Response Compression

- [ ] Add `GZipMiddleware` that compresses responses based on `Accept-Encoding`
- [ ] Configurable minimum response size threshold
- [ ] Support gzip and/or brotli
- [ ] Skip compression for streaming responses

### 7. Trusted Host / HTTPS Redirect

- [ ] Add `TrustedHostMiddleware` that validates `Host` header
- [ ] Add `HTTPSRedirectMiddleware` that redirects HTTP to HTTPS
- [ ] Configurable allowed hosts list

### 8. Client IP Address

- [ ] Expose `request.client.host` on the `Request` object
- [ ] Extract from `hyper`'s connected socket info
- [ ] Support `X-Forwarded-For` / `X-Real-IP` behind reverse proxy (configurable)

### 9. Response Cookies API

- [x] Add `response.set_cookie(name, value, max_age, path, domain, httponly, secure, samesite)`
- [ ] Add `response.delete_cookie(name, path, domain)`
- [x] Type-safe API instead of manual `insert_header("set-cookie", "...")`

### 13. OAuth2 / Security Utilities

- [ ] Add `OAuth2PasswordBearer(tokenUrl="/token")` dependency
- [ ] Add `HTTPBasic` dependency for HTTP Basic auth
- [ ] Add `APIKeyHeader` / `APIKeyQuery` dependencies
- [ ] Support OAuth2 scopes

### 14 Content Negotiation

- [ ] Inspect `Accept` header to determine response format
- [ ] Support multiple serializers per route (JSON, XML, MessagePack)
- [ ] Default to JSON, fallback based on client preference

---

## Bug Fixes & Quality

### Stubs

- [x] Fix `Session()` return type in `__init__.pyi` — should be `Callable`, not `Response`
- [x] Remove `catcher` from `__init__.py.__all__` (doesn't exist)
- [x] Remove `"from typing_extensions import Self"` from stub `__all__`
- [x] Fix docstrings: `app_data()`, `attach()`, `wrap()` say `Returns: None` but return `self`

### Security

- [x] Change `SameSite=Lax` to `SameSite=Strict` on session cookies (or make configurable)
- [ ] Add `Origin` / `Referer` header check in session middleware for state-changing methods
- [x] Replace `unwrap()` in `insert_header` / `append_header` with proper error handling

### Performance

- [x] Cache `Regex::new` in `parse_params_value` (slug parsing) — currently recompiles every call
- [ ] Evaluate middleware chain `py.eval()` overhead — consider alternatives

### Safety

- [x] Replace `unsafe { std::mem::transmute }`

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
