use hyper::{body::Bytes, header::CONTENT_TYPE, HeaderMap};
use pyo3::{basic::CompareOp, prelude::*, IntoPyObjectExt};

use crate::response::Response;

/// HTTP status codes enumeration.
///
/// This enum contains standard HTTP status codes as defined in RFC 7231 and other RFCs.
/// Status codes are grouped by their first digit:
/// - 1xx: Informational responses
/// - 2xx: Successful responses
/// - 3xx: Redirection messages
/// - 4xx: Client error responses
/// - 5xx: Server error responses
///
/// Example:
/// ```python
/// from oxapy import Status, Response
///
/// # Create a not found response
/// response = Response("Not found", status=Status.NOT_FOUND)
///
/// # Check status in a handler
/// @router.get("/resource/{id}")
/// def get_resource(request, id):
///     resource = find_resource(id)
///     if resource is None:
///         return Status.NOT_FOUND
///     return resource
/// ```
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[pyclass]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum Status {
    /// 100 Continue - Server has received the request headers and client should proceed to send the request body
    CONTINUE = 100,
    /// 101 Switching Protocols - Server is switching protocols as requested by the client
    SWITCHING_PROTOCOLS = 101,
    /// 102 Processing - Server has received and is processing the request, but no response is available yet
    PROCESSING = 102,

    /// 200 OK - Request has succeeded
    OK = 200,
    /// 201 Created - Request has succeeded and a new resource has been created
    CREATED = 201,
    /// 202 Accepted - Request has been accepted for processing, but processing has not been completed
    ACCEPTED = 202,
    /// 203 Non-Authoritative Information - Request was successful but returned metadata from another source
    NON_AUTHORITATIVE_INFORMATION = 203,
    /// 204 No Content - Request succeeded but returns no message body
    NO_CONTENT = 204,
    /// 205 Reset Content - Request succeeded, and the user agent should reset the document view
    RESET_CONTENT = 205,
    /// 206 Partial Content - Server is delivering only part of the resource due to a range header in the request
    PARTIAL_CONTENT = 206,
    /// 207 Multi-Status - Provides status for multiple independent operations (WebDAV)
    MULTI_STATUS = 207,
    /// 208 Already Reported - Used in a DAV response to avoid enumerating members of multiple bindings to the same collection
    ALREADY_REPORTED = 208,
    /// 226 IM Used - The server has fulfilled a request for the resource, and the response is a representation of the instance-manipulation result
    IM_USED = 226,
    /// 300 Multiple Choices - The request has more than one possible response
    MULTIPLE_CHOICES = 300,
    /// 301 Moved Permanently - The URI of the requested resource has been changed permanently
    MOVED_PERMANENTLY = 301,
    /// 302 Found - The URI of the requested resource has been changed temporarily
    FOUND = 302,
    /// 303 See Other - The response to the request can be found at another URI
    SEE_OTHER = 303,
    /// 304 Not Modified - Resource hasn't been modified since last request
    NOT_MODIFIED = 304,
    /// 305 Use Proxy - The requested resource must be accessed through the proxy given by the location field
    USE_PROXY = 305,
    /// 307 Temporary Redirect - The request should be repeated with another URI, but future requests can still use the original URI
    TEMPORARY_REDIRECT = 307,
    /// 308 Permanent Redirect - All future requests should use another URI
    PERMANENT_REDIRECT = 308,

    /// 400 Bad Request - The server cannot or will not process the request due to client error
    BAD_REQUEST = 400,
    /// 401 Unauthorized - Authentication is required and has failed or not been provided
    UNAUTHORIZED = 401,
    /// 402 Payment Required - Reserved for future use
    PAYMENT_REQUIRED = 402,
    /// 403 Forbidden - Server understood the request but refuses to authorize it
    FORBIDDEN = 403,
    /// 404 Not Found - The requested resource could not be found on the server
    NOT_FOUND = 404,
    /// 405 Method Not Allowed - The request method is not supported for the requested resource
    METHOD_NOT_ALLOWED = 405,
    /// 406 Not Acceptable - The requested resource is capable of generating only content not acceptable according to the Accept headers
    NOT_ACCEPTABLE = 406,
    /// 407 Proxy Authentication Required - Authentication with the proxy is required
    PROXY_AUTHENTICATION_REQUIRED = 407,
    /// 408 Request Timeout - The server timed out waiting for the request
    REQUEST_TIMEOUT = 408,
    /// 409 Conflict - The request could not be completed due to a conflict with the current state of the resource
    CONFLICT = 409,
    /// 410 Gone - The requested resource is no longer available and will not be available again
    GONE = 410,
    /// 411 Length Required - The request did not specify the length of its content, which is required
    LENGTH_REQUIRED = 411,
    /// 412 Precondition Failed - Server does not meet one of the preconditions in the request
    PRECONDITION_FAILED = 412,
    /// 413 Payload Too Large - The request is larger than the server is willing or able to process
    PAYLOAD_TOO_LARGE = 413,
    /// 414 URI Too Long - The URI provided was too long for the server to process
    URI_TOO_LONG = 414,
    /// 415 Unsupported Media Type - The request entity has a media type which the server does not support
    UNSUPPORTED_MEDIA_TYPE = 415,
    /// 416 Range Not Satisfiable - The client has asked for a portion of the file, but the server cannot supply that portion
    RANGE_NOT_SATISFIABLE = 416,
    /// 417 Expectation Failed - The server cannot meet the requirements of the Expect request-header field
    EXPECTATION_FAILED = 417,
    /// 418 I'm a Teapot - The server refuses the attempt to brew coffee with a teapot (April Fools' joke)
    IM_A_TEAPOT = 418,

    /// 421 Misdirected Request - The request was directed at a server that is not able to produce a response
    MISDIRECTED_REQUEST = 421,
    /// 422 Unprocessable Entity - The request was well-formed but was unable to be followed due to semantic errors
    UNPROCESSABLE_ENTITY = 422,
    /// 423 Locked - The resource that is being accessed is locked (WebDAV)
    LOCKED = 423,
    /// 424 Failed Dependency - The request failed due to failure of a previous request (WebDAV)
    FAILED_DEPENDENCY = 424,

    /// 426 Upgrade Required - The client should switch to a different protocol
    UPGRADE_REQUIRED = 426,

    /// 428 Precondition Required - The origin server requires the request to be conditional
    PRECONDITION_REQUIRED = 428,
    /// 429 Too Many Requests - The user has sent too many requests in a given amount of time
    TOO_MANY_REQUESTS = 429,

    /// 431 Request Header Fields Too Large - The server is unwilling to process the request because its header fields are too large
    REQUEST_HEADER_FIELDS_TOO_LARGE = 431,

    /// 451 Unavailable For Legal Reasons - The server is denying access to the resource as a consequence of a legal demand
    UNAVAILABLE_FOR_LEGAL_REASONS = 451,

    /// 500 Internal Server Error - The server has encountered a situation it doesn't know how to handle
    INTERNAL_SERVER_ERROR = 500,
    /// 501 Not Implemented - The server does not support the functionality required to fulfill the request
    NOT_IMPLEMENTED = 501,
    /// 502 Bad Gateway - The server was acting as a gateway or proxy and received an invalid response from the upstream server
    BAD_GATEWAY = 502,
    /// 503 Service Unavailable - The server is not ready to handle the request, often due to maintenance or overloading
    SERVICE_UNAVAILABLE = 503,
    /// 504 Gateway Timeout - The server was acting as a gateway or proxy and did not receive a timely response from the upstream server
    GATEWAY_TIMEOUT = 504,
    /// 505 HTTP Version Not Supported - The server does not support the HTTP protocol version used in the request
    HTTP_VERSION_NOT_SUPPORTED = 505,
    /// 506 Variant Also Negotiates - The server has an internal configuration error: the chosen variant resource is configured to engage in transparent content negotiation itself
    VARIANT_ALSO_NEGOTIATES = 506,
    /// 507 Insufficient Storage - The server is unable to store the representation needed to complete the request (WebDAV)
    INSUFFICIENT_STORAGE = 507,
    /// 508 Loop Detected - The server detected an infinite loop while processing the request (WebDAV)
    LOOP_DETECTED = 508,

    /// 510 Not Extended - Further extensions to the request are required for the server to fulfill it
    NOT_EXTENDED = 510,
    /// 511 Network Authentication Required - The client needs to authenticate to gain network access
    NETWORK_AUTHENTICATION_REQUIRED = 511,
}

#[pymethods]
impl Status {
    /// Compare two Status objects.
    ///
    /// This method allows Python code to use comparison operators (==, !=, <, <=, >, >=)
    /// between Status enum values.
    ///
    /// Args:
    ///     other (Status): The status to compare with.
    ///     op (CompareOp): The comparison operation to perform.
    ///
    /// Returns:
    ///     bool: The result of the comparison.
    ///
    /// Example:
    /// ```python
    /// # Check if a status code is a success code (2xx)
    /// if status >= Status.OK and status < Status.MULTIPLE_CHOICES:
    ///     print("Success!")
    /// ```
    fn __richcmp__(
        &self,
        other: PyRef<Status>,
        op: CompareOp,
        py: Python<'_>,
    ) -> PyResult<PyObject> {
        let lhs = *self as u16;
        let rhs = *other as u16;
        let result = match op {
            CompareOp::Eq => lhs == rhs,
            CompareOp::Ne => lhs != rhs,
            CompareOp::Lt => lhs < rhs,
            CompareOp::Le => lhs <= rhs,
            CompareOp::Gt => lhs > rhs,
            CompareOp::Ge => lhs >= rhs,
        };
        result.into_py_any(py)
    }
}

impl From<Status> for Response {
    fn from(val: Status) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "text/plain".parse().unwrap());
        Response {
            status: val,
            headers,
            body: Bytes::new(),
        }
    }
}
