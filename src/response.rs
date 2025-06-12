use std::collections::HashMap;
use std::str;

use hyper::body::Bytes;
use pyo3::{prelude::*, types::PyBytes};

use crate::{
    json,
    session::{Session, SessionStore},
    status::Status,
    IntoPyException,
};

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
    #[pyo3(get, set)]
    pub headers: HashMap<String, String>,
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

        Ok(Self {
            status,
            body,
            headers: [("Content-Type".to_string(), content_type.to_string())].into(),
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
    pub fn insert_header(&mut self, key: &str, value: String) -> Self {
        self.headers.insert(key.to_string(), value);
        self.clone()
    }
}

impl Response {
    pub fn set_body(mut self, body: String) -> Self {
        self.body = body.into();
        self
    }

    pub fn set_session_cookie(&mut self, session: &Session, store: &SessionStore) {
        let cookie_header = store.get_cookie_header(session);
        self.headers.insert("Set-Cookie".to_string(), cookie_header);
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
        (
            Self,
            Response {
                status: Status::MOVED_PERMANENTLY,
                body: Bytes::new(),
                headers: HashMap::from([
                    ("Content-Type".to_string(), "text/html".to_string()),
                    ("Location".to_string(), location.to_string()),
                ]),
            },
        )
    }
}
