use std::str;

use hyper::{
    body::Bytes,
    header::{HeaderName, CONTENT_TYPE, LOCATION},
    HeaderMap,
};
use pyo3::{prelude::*, types::PyBytes};

use crate::{json, status::Status, IntoPyException};

/// HTTP response object that is returned from request handlers.
///
/// Args:
///     body (any): The response body, can be a string, bytes, or JSON-serializable object.
///     status (Status, optional): The HTTP status code (defaults to Status.OK).
///     content_type (str, optional): The content type header (defaults to "application/json").
///
/// Returns:
///     Response: A new HTTP response.
///
/// Example:
/// ```python
/// # JSON response
/// response = Response({"message": "Success"})
///
/// # Plain text response
/// response = Response("Hello, World!", content_type="text/plain")
///
/// # HTML response with custom status
/// response = Response("<h1>Not Found</h1>", Status.NOT_FOUND, "text/html")
/// ```
#[derive(Clone)]
#[pyclass(subclass)]
pub struct Response {
    #[pyo3(get, set)]
    pub status: Status,
    pub body: Bytes,
    pub headers: HeaderMap,
}

#[pymethods]
impl Response {
    /// Create a new Response instance.
    ///
    /// Args:
    ///     body (any): The response body content (string, bytes, or JSON-serializable object).
    ///     status (Status, optional): HTTP status code, defaults to Status.OK.
    ///     content_type (str, optional): Content-Type header, defaults to "application/json".
    ///
    /// Returns:
    ///     Response: A new response object.
    ///
    /// Example:
    /// ```python
    /// # Return JSON
    /// response = Response({"message": "Hello"})
    ///
    /// # Return plain text
    /// response = Response("Hello", content_type="text/plain")
    ///
    /// # Return error
    /// response = Response("Not authorized", status=Status.UNAUTHORIZED)
    /// ```
    #[new]
    #[pyo3(signature=(body, status = Status::OK , content_type="application/json"))]
    pub fn new(
        body: PyObject,
        status: Status,
        content_type: &str,
        py: Python<'_>,
    ) -> PyResult<Self> {
        let body = if let Ok(bytes) = body.extract::<Py<PyBytes>>(py) {
            bytes.as_bytes(py).to_vec().into()
        } else if content_type == "application/json" {
            json::dumps(&body)?.into()
        } else {
            body.to_string().into()
        };

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, content_type.parse().into_py_exception()?);

        Ok(Self {
            status,
            body,
            headers,
        })
    }

    /// Get the response body as a string.
    ///
    /// Returns:
    ///     str: The response body as a UTF-8 string.
    ///
    /// Raises:
    ///     Exception: If the body cannot be converted to a valid UTF-8 string.
    #[getter]
    fn body(&self) -> PyResult<String> {
        Ok(str::from_utf8(&self.body).into_py_exception()?.to_string())
    }

    /// Get the response headers as a list of key-value tuples.
    ///
    /// Returns:
    ///
    ///     list[tuple[str, str]]: The list of headers in the response.
    ///
    /// Raises:
    ///
    ///     Exception: If a header value cannot be converted to a valid UTF-8 string.
    ///
    /// Example:
    /// ```python
    /// response = Response("Hello")
    /// headers = response.headers
    /// for name, value in headers:
    ///     print(f"{name}: {value}")
    /// ```
    #[getter]
    fn headers(&self) -> Vec<(&str, &str)> {
        self.headers
            .iter()
            .map(|(k, v)| (k.as_str(), v.to_str().unwrap()))
            .collect()
    }

    /// Add or update a header in the response.
    ///
    /// Args:
    ///     key (str): The header name.
    ///     value (str): The header value.
    ///
    /// Returns:
    ///     Response: The response instance (for method chaining).
    ///
    /// Example:
    /// ```python
    /// response = Response("Hello")
    /// response.insert_header("Cache-Control", "no-cache")
    /// ```
    pub fn insert_header(&mut self, key: &str, value: String) {
        self.headers.insert(
            HeaderName::from_bytes(key.as_bytes()).unwrap(),
            value.parse().unwrap(),
        );
    }

    /// Append a header to the response.
    ///
    /// This is useful for headers that can appear multiple times, such as `Set-Cookie`.
    ///
    /// Args:
    ///
    ///     key (str): The header name.
    ///     value (str): The header value.
    ///
    /// Returns:
    ///
    ///     None
    ///
    /// Example:
    /// ```python
    /// response = Response("Hello")
    /// response.insert_header("Set-Cookie", "sessionid=abc123")
    /// response.append_header("Set-Cookie", "theme=dark")
    /// ```
    pub fn append_header(&mut self, key: &str, value: String) {
        self.headers.append(
            HeaderName::from_bytes(key.as_bytes()).unwrap(),
            value.parse().unwrap(),
        );
    }
}

impl Response {
    pub fn set_body(mut self, body: String) -> Self {
        self.body = body.into();
        self
    }

    pub fn insert_or_append_cookie(&mut self, cookie_header: String) {
        if self.headers.contains_key("Set-Cookie") {
            self.append_header("Set-Cookie", cookie_header);
        } else {
            self.insert_header("Set-Cookie", cookie_header);
        }
    }
}

/// HTTP redirect response.
///
/// A specialized response type that redirects the client to a different URL.
///
/// Args:
///     location (str): The URL to redirect to.
///
/// Returns:
///     Redirect: A redirect response.
///
/// Example:
/// ```python
/// # Redirect to the home page
/// return Redirect("/home")
///
/// # Redirect to an external site
/// return Redirect("https://example.com")
/// ```
#[pyclass(subclass, extends=Response)]
pub struct Redirect;

#[pymethods]
impl Redirect {
    /// Create a new HTTP redirect response.
    ///
    /// Args:
    ///     location (str): The URL to redirect to.
    ///
    /// Returns:
    ///     Redirect: A redirect response with status 301 (Moved Permanently).
    ///
    /// Example:
    /// ```python
    /// # Redirect user after form submission
    /// @router.post("/submit")
    /// def submit_form(request):
    ///     # Process form...
    ///     return Redirect("/thank-you")
    /// ```
    #[new]
    fn new(location: String) -> (Self, Response) {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "text/html".parse().unwrap());
        headers.insert(LOCATION, location.parse().unwrap());
        (
            Self,
            Response {
                status: Status::MOVED_PERMANENTLY,
                body: Bytes::new(),
                headers,
            },
        )
    }
}
