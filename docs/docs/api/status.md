# Status

`Status` is an enum of HTTP status codes, named after the standard reason phrases.

## Usage

Return a `Status` directly from a handler for an error response with an empty JSON body:

```python
from oxapy import Status, get


@get("/error")
def error(request):
    return Status.INTERNAL_SERVER_ERROR
```

Use it as the `status` argument of `Response` or in a `(body, Status)` tuple:

```python
from oxapy import Response, Status

Response("Not found", status=Status.NOT_FOUND)
return ("Created", Status.CREATED)
```

## Code and comparisons

`status.code()` returns the numeric code:

```python
Status.NOT_FOUND.code()  # 404
```

Status values support comparison operators, including ranges:

```python
status = response.status

if status == Status.OK:
    print("success")

if status >= Status.OK and status < Status.MULTIPLE_CHOICES:
    print("2xx success")
```

## Common members

| Member | Code |
| --- | --- |
| `CONTINUE` | 100 |
| `OK` | 200 |
| `CREATED` | 201 |
| `ACCEPTED` | 202 |
| `NO_CONTENT` | 204 |
| `PARTIAL_CONTENT` | 206 |
| `MULTIPLE_CHOICES` | 300 |
| `MOVED_PERMANENTLY` | 301 |
| `FOUND` | 302 |
| `NOT_MODIFIED` | 304 |
| `TEMPORARY_REDIRECT` | 307 |
| `PERMANENT_REDIRECT` | 308 |
| `BAD_REQUEST` | 400 |
| `UNAUTHORIZED` | 401 |
| `PAYMENT_REQUIRED` | 402 |
| `FORBIDDEN` | 403 |
| `NOT_FOUND` | 404 |
| `METHOD_NOT_ALLOWED` | 405 |
| `NOT_ACCEPTABLE` | 406 |
| `REQUEST_TIMEOUT` | 408 |
| `CONFLICT` | 409 |
| `GONE` | 410 |
| `PAYLOAD_TOO_LARGE` | 413 |
| `URI_TOO_LONG` | 414 |
| `UNSUPPORTED_MEDIA_TYPE` | 415 |
| `RANGE_NOT_SATISFIABLE` | 416 |
| `EXPECTATION_FAILED` | 417 |
| `IM_A_TEAPOT` | 418 |
| `UNPROCESSABLE_ENTITY` | 422 |
| `LOCKED` | 423 |
| `TOO_MANY_REQUESTS` | 429 |
| `INTERNAL_SERVER_ERROR` | 500 |
| `NOT_IMPLEMENTED` | 501 |
| `BAD_GATEWAY` | 502 |
| `SERVICE_UNAVAILABLE` | 503 |
| `GATEWAY_TIMEOUT` | 504 |
| `HTTP_VERSION_NOT_SUPPORTED` | 505 |

The enum also includes the less common codes: `SWITCHING_PROTOCOLS` (101), `PROCESSING` (102), `RESET_CONTENT` (205), `MULTI_STATUS` (207), `ALREADY_REPORTED` (208), `IM_USED` (226), `USE_PROXY` (305), `PROXY_AUTHENTICATION_REQUIRED` (407), `LENGTH_REQUIRED` (411), `PRECONDITION_FAILED` (412), `MISDIRECTED_REQUEST` (421), `FAILED_DEPENDENCY` (424), `UPGRADE_REQUIRED` (426), `PRECONDITION_REQUIRED` (428), `REQUEST_HEADER_FIELDS_TOO_LARGE` (431), `UNAVAILABLE_FOR_LEGAL_REASONS` (451), `VARIANT_ALSO_NEGOTIATES` (506), `INSUFFICIENT_STORAGE` (507), `LOOP_DETECTED` (508), `NOT_EXTENDED` (510), and `NETWORK_AUTHENTICATION_REQUIRED` (511).

## Related

- [Responses guide](../guides/responses) — returning status codes from handlers
- [Error Handling guide](../guides/error-handling) — raising typed exceptions
- [Response](./response) — building responses
