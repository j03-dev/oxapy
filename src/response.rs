use crate::{json, status::Status, IntoPyException};
use futures_util::StreamExt;
use hyper::body::Frame;
use hyper::http::HeaderValue;
use hyper::{
    body::Bytes,
    header::{HeaderName, CONTENT_TYPE, LOCATION},
    HeaderMap,
};

use futures_util::stream;
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::header::CACHE_CONTROL;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyIterator, PyString};
use std::convert::Infallible;
use std::str;
use std::sync::Arc;

pub type Body = http_body_util::combinators::BoxBody<Bytes, Infallible>;

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
/// `
#[pyclass(subclass)]
#[derive(Clone)]
pub struct Response {
    #[pyo3(get, set)]
    pub status: Status,
    pub body: Arc<Body>,
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
    pub fn new(body: Bound<PyAny>, status: Status, content_type: &str) -> PyResult<Self> {
        let content_type = HeaderValue::from_str(content_type).into_py_exception()?;

        if body.is_instance_of::<PyString>() {
            return Self::from_str(body.to_string(), status, content_type);
        }

        if content_type == "application/json" {
            return Self::from_json(body, status, content_type);
        }

        if body.is_instance_of::<PyBytes>() {
            return Self::from_bytes(body.extract()?, status, content_type);
        }

        if body.is_instance_of::<PyIterator>() {
            return Self::from_stream(body, status, content_type);
        }

        Err(PyTypeError::new_err("Unsupported response type"))
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
        todo!()
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
        self.body = Arc::new(Full::new(body.into()).boxed());
        self
    }

    pub fn insert_or_append_cookie(&mut self, cookie_header: String) {
        if self.headers.contains_key("Set-Cookie") {
            self.append_header("Set-Cookie", cookie_header);
        } else {
            self.insert_header("Set-Cookie", cookie_header);
        }
    }

    fn from_str(s: String, status: Status, content_type: HeaderValue) -> PyResult<Self> {
        Ok(Self {
            body: Arc::new(Full::new(s.into()).boxed()),
            status,
            headers: HeaderMap::from_iter([(CONTENT_TYPE, content_type)]),
        })
    }

    fn from_bytes(b: &[u8], status: Status, content_type: HeaderValue) -> PyResult<Self> {
        Ok(Self {
            status,
            body: Arc::new(Full::new(Bytes::copy_from_slice(b)).boxed()),
            headers: HeaderMap::from_iter([(CONTENT_TYPE, content_type)]),
        })
    }

    fn from_json(obj: Bound<PyAny>, status: Status, content_type: HeaderValue) -> PyResult<Self> {
        let json = json::dumps(&obj.into())?;
        Ok(Self {
            status,
            body: Arc::new(Full::new(json.into()).boxed()),
            headers: HeaderMap::from_iter([(CONTENT_TYPE, content_type)]),
        })
    }

    fn from_stream(
        obj: Bound<PyAny>,
        status: Status,
        content_type: HeaderValue,
    ) -> PyResult<Response> {
        let mut chunks: Vec<Vec<u8>> = Vec::new();

        for item in obj.try_iter()? {
            chunks.push(item?.extract()?);
        }

        let stream = stream::iter(chunks).map(|it| Ok(Frame::data(Bytes::from(it))));

        let body = StreamBody::new(Box::pin(stream));

        let mut headers = HeaderMap::default();
        headers.insert(CONTENT_TYPE, content_type);
        headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));

        Ok(Response {
            status,
            body: Arc::new(BodyExt::boxed(body)),
            headers,
        })
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
    fn new(location: String) -> (Redirect, Response) {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "text/html".parse().unwrap());
        headers.insert(LOCATION, location.parse().unwrap());
        (
            Self,
            Response {
                status: Status::MOVED_PERMANENTLY,
                body: Arc::new(Full::new(Bytes::new()).boxed()),
                headers,
            },
        )
    }
}

impl TryFrom<Response> for hyper::Response<Body> {
    type Error = hyper::http::Error;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        let mut builder = hyper::Response::builder().status(response.status as u16);
        for (name, value) in response.headers.iter() {
            builder = builder.header(name, value);
        }

        let body = match Arc::try_unwrap(response.body) {
            Ok(b) => b,
            Err(_) => panic!("failed to unwrap arc"),
        };

        builder.body(body)
    }
}
