use std::convert::Infallible;
use std::fs;
use std::io::Read;
use std::str::{self, FromStr};
use std::sync::Arc;

use futures_util::{StreamExt, stream};
use http_body_util::{BodyExt, Full, StreamBody, combinators::BoxBody};
use hyper::{
    HeaderMap,
    body::{Bytes, Frame},
    header::{CACHE_CONTROL, CONTENT_TYPE, HeaderName, LOCATION},
    http::HeaderValue,
};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString};
use pyo3_stub_gen::derive::*;

use crate::cors::Cors;
use crate::{IntoPyException, ProcessRequest, Status, convert_to_response, json};

pub type Body = BoxBody<Bytes, Infallible>;

#[derive(Clone)]
pub enum ResponseBody {
    Bytes(Bytes),
    Stream(Arc<Body>),
}

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
#[gen_stub_pyclass]
#[pyclass(from_py_object, subclass)]
#[derive(Clone)]
pub struct Response {
    #[pyo3(get, set)]
    pub status: Status,
    pub body: ResponseBody,
    pub headers: HeaderMap,
}

#[gen_stub_pymethods]
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
    #[gen_stub(override_return_type(type_repr = "typing_extensions.Self", imports = ("typing_extensions",)))]
    pub fn new(body: Bound<PyAny>, status: Status, content_type: &str) -> PyResult<Self> {
        let content_type = HeaderValue::from_str(content_type).into_py_exception()?;

        if content_type == "application/json" {
            return Self::from_json(body, status, content_type);
        }

        if body.is_instance_of::<PyBytes>() {
            return Self::from_bytes(body.extract()?, status, content_type);
        }

        if body.is_instance_of::<PyString>() {
            return Self::from_str(body.to_string(), status, content_type);
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
        match &self.body {
            ResponseBody::Bytes(b) => {
                let s = str::from_utf8(b).into_py_exception()?;
                Ok(s.to_string())
            }
            _ => {
                let message = "response body is streaming and cannot be extracted as a string";
                Err(PyTypeError::new_err(message))
            }
        }
    }

    /// Get the response headers as a list of key-value tuples.
    ///
    /// Returns:
    ///     list[tuple[str, str]]: The list of headers in the response.
    ///
    /// Raises:
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
        // we return vec of tuple over dictionary because,
        // dict can't store diff value with same key
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
    pub fn insert_header(&mut self, key: &str, value: &str) -> PyResult<()> {
        let header_name = HeaderName::from_str(key).into_py_exception()?;
        let header_value = HeaderValue::from_str(value).into_py_exception()?;
        self.headers.insert(header_name, header_value);
        Ok(())
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
    pub fn append_header(&mut self, key: &str, value: &str) -> PyResult<()> {
        let header_name = HeaderName::from_str(key).into_py_exception()?;
        let header_value = HeaderValue::from_str(value).into_py_exception()?;
        self.headers.append(header_name, header_value);
        Ok(())
    }

    /// Add a cookie to the response via the ``Set-Cookie`` header.
    ///
    /// Builds a ``Set-Cookie`` header from the provided parameters and inserts
    /// it into the response. If a ``Set-Cookie`` header already exists the new
    /// cookie is appended (allowing multiple cookies).
    ///
    /// Args:
    ///     name (str): The cookie name.
    ///     value (str): The cookie value.
    ///     max_age (int, optional): Max-Age in seconds. Defaults to 3600.
    ///     path (str, optional): Path attribute. Defaults to ``"/"``.
    ///     domain (str, optional): Domain attribute. Empty string means omitted.
    ///     httponly (bool, optional): HttpOnly flag. Defaults to ``True``.
    ///     secure (bool, optional): Secure flag. Defaults to ``True``.
    ///     samesite (str, optional): SameSite attribute. Defaults to ``"Lax"``.
    ///
    /// Returns:
    ///     None
    ///
    /// Example:
    /// ```python
    /// response = Response("OK")
    /// response.set_cookie("session", "abc123", max_age=3600, httponly=True, secure=True)
    /// response.set_cookie("theme", "dark", max_age=86400)
    /// ```
    #[pyo3(signature=(name, value, max_age=3600, path="/", domain="", httponly=true, secure=true, samesite="Lax"))]
    pub fn set_cookie(
        &mut self,
        name: &str,
        value: &str,
        max_age: i64,
        path: &str,
        domain: &str,
        httponly: bool,
        secure: bool,
        samesite: &str,
    ) -> PyResult<()> {
        let mut cookie_header =
            format!("{name}={value}; Path={path}; Max-Age={max_age}; SameSite={samesite}");

        if !domain.is_empty() {
            cookie_header.push_str(&format!("; Domain={domain}"));
        }
        if httponly {
            cookie_header.push_str("; HttpOnly");
        }
        if secure {
            cookie_header.push_str("; Secure");
        }

        if self.headers.contains_key("Set-Cookie") {
            self.append_header("Set-Cookie", &cookie_header)?;
        } else {
            self.insert_header("Set-Cookie", &cookie_header)?;
        }

        Ok(())
    }
}

impl Response {
    pub fn set_body(mut self, body: String) -> Self {
        self.body = ResponseBody::Bytes(Bytes::from(body));
        self
    }

    fn from_str(s: String, status: Status, content_type: HeaderValue) -> PyResult<Self> {
        Ok(Self {
            body: ResponseBody::Bytes(Bytes::from(s)),
            status,
            headers: HeaderMap::from_iter([(CONTENT_TYPE, content_type)]),
        })
    }

    fn from_bytes(b: &[u8], status: Status, content_type: HeaderValue) -> PyResult<Self> {
        Ok(Self {
            status,
            body: ResponseBody::Bytes(Bytes::copy_from_slice(b)),
            headers: HeaderMap::from_iter([(CONTENT_TYPE, content_type)]),
        })
    }

    fn from_json(obj: Bound<PyAny>, status: Status, content_type: HeaderValue) -> PyResult<Self> {
        Ok(Self {
            status,
            body: ResponseBody::Bytes(json::dumps(&obj)?.into()),
            headers: HeaderMap::from_iter([(CONTENT_TYPE, content_type)]),
        })
    }

    pub(crate) fn call_wrapper(mut self, pr: &ProcessRequest) -> Self {
        if let Some(wrapper) = &pr.wrapper {
            let request = (*pr.request).clone();
            self = Python::attach(|py| {
                let result = wrapper.call(py, (request, self), None)?;
                convert_to_response(result, py)
            })
            .unwrap_or_else(Response::from);
        }
        self
    }

    pub(crate) fn apply_cors(mut self, cors: &Option<Arc<Cors>>) -> PyResult<Self> {
        if let Some(cors) = cors {
            cors.apply_headers(&mut self)?;
        }
        Ok(self)
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
#[gen_stub_pyclass]
#[pyclass(subclass, extends=Response)]
pub struct Redirect;

#[gen_stub_pymethods]
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
    /// from oxapy import post, Redirect
    ///
    /// # Redirect user after form submission
    /// @post("/submit")
    /// def submit_form(request):
    ///     # Process form...
    ///     return Redirect("/thank-you")
    /// ```
    #[new]
    #[gen_stub(override_return_type(type_repr = "typing_extensions.Self", imports = ("typing_extensions",)))]
    fn new(location: String) -> PyResult<PyClassInitializer<Self>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
        headers.insert(LOCATION, location.parse().into_py_exception()?);
        Ok(PyClassInitializer::from(Response {
            status: Status::MOVED_PERMANENTLY,
            body: ResponseBody::Bytes(Bytes::new()),
            headers,
        })
        .add_subclass(Self))
    }
}

struct ChunkIter {
    file: fs::File,
    buf_size: usize,
}

impl Iterator for ChunkIter {
    type Item = Bytes;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = vec![0; self.buf_size];
        match self.file.read(&mut buf) {
            Ok(0) => None,
            Ok(n) => {
                buf.truncate(n);
                Some(Bytes::from(buf))
            }
            Err(_) => None,
        }
    }
}

/// HTTP file streaming response for efficiently serving large files.
///
/// FileStreaming provides an efficient way to serve files by streaming them in chunks
/// rather than loading the entire file into memory. This is particularly useful for
/// large files like videos, images, or documents.
///
/// The file is read in configurable buffer chunks and streamed to the client,
/// allowing for low memory usage regardless of file size.
///
/// Args:
///     path (str): The file system path to the file to be streamed.
///     buf_size (int, optional): The buffer size in bytes for reading chunks. Defaults to 8192.
///     status (Status, optional): HTTP status code. Defaults to Status.OK.
///     content_type (str, optional): MIME type of the file. Defaults to "application/octet-stream".
///
/// Returns:
///     tuple[FileStreaming, Response]: A tuple containing the FileStreaming instance and Response.
///
/// Raises:
///     OSError: If the file cannot be opened or read.
///     ValueError: If the path is invalid or inaccessible.
///
/// Example:
/// ```python
/// from oxapy import Router, FileStreaming, Status, get
///
/// router = Router()
///
/// # Stream a video file
/// @get("/videos/{*path}")
/// def serve_video(request, path):
///     return FileStreaming(
///         f"./media/videos/{path}",
///         buf_size=16384,  # 16KB chunks
///         content_type="video/mp4"
///     )
///
/// # Stream static files
/// @get("/static/{*path}")
/// def serve_static(request, path):
///     return FileStreaming(
///         f"./static/{path}",
///         buf_size=32768,  # 32KB chunks for better performance
///         content_type="application/octet-stream"
///     )
///
/// # Stream with custom status for partial content
/// @get("/downloads/{filename}")
/// def serve_download(request, filename):
///     return FileStreaming(
///         f"./downloads/{filename}",
///         status=Status.PARTIAL_CONTENT,
///         content_type="application/pdf"
///     )
/// ```
#[gen_stub_pyclass]
#[pyclass(subclass, extends=Response)]
pub struct FileStreaming;

#[gen_stub_pymethods]
#[pymethods]
impl FileStreaming {
    /// Create a new FileStreaming response.
    ///
    /// Opens the specified file and prepares it for streaming to the client.
    /// The file is read in chunks of the specified buffer size, which helps
    /// control memory usage and allows for efficient streaming of large files.
    ///
    /// Args:
    ///     path (str): Path to the file to stream. Must be accessible and readable.
    ///     buf_size (int, optional): Buffer size in bytes for reading file chunks.
    ///                              Larger values may improve performance for large files
    ///                              but use more memory. Defaults to 8192 bytes (8KB).
    ///     status (Status, optional): HTTP status code for the response. Defaults to Status.OK.
    ///     content_type (str, optional): MIME type of the file content.
    ///                                  Defaults to "application/octet-stream".
    ///
    /// Returns:
    ///     FileStreaming: A FileStreaming instance.
    ///
    /// Raises:
    ///     OSError: If the file at the specified path cannot be opened or read.
    ///     PermissionError: If the process lacks permission to read the file.
    ///
    /// Note:
    ///     The response automatically includes "Cache-Control: no-cache" header
    ///     to prevent caching of streamed content.
    ///
    /// Example:
    /// ```python
    /// # Basic file streaming
    /// streaming_response = FileStreaming("./files/document.pdf")
    ///
    /// # Custom buffer size for better performance
    /// streaming_response = FileStreaming(
    ///     "./media/large-video.mp4",
    ///     buf_size=65536,  # 64KB chunks
    ///     content_type="video/mp4"
    /// )
    /// ```
    #[new]
    #[pyo3(signature=(path, buf_size=8192, status=Status::OK, content_type="application/octet-stream"))]
    #[gen_stub(override_return_type(type_repr = "typing_extensions.Self", imports = ("typing_extensions",)))]
    fn new(
        path: &str,
        buf_size: usize,
        status: Status,
        content_type: &str,
    ) -> PyResult<PyClassInitializer<Self>> {
        let file = fs::File::open(path)?;
        let chunk_iter = ChunkIter { file, buf_size };
        let stream = stream::iter(chunk_iter).map(|bytes| Ok(Frame::data(bytes)));
        let body = StreamBody::new(Box::pin(stream));
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, content_type.parse().into_py_exception()?);
        headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        Ok(PyClassInitializer::from(Response {
            status,
            body: ResponseBody::Stream(Arc::new(BodyExt::boxed(body))),
            headers,
        })
        .add_subclass(Self))
    }
}

impl TryFrom<Response> for hyper::Response<Body> {
    type Error = hyper::http::Error;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        let mut builder = hyper::Response::builder().status(response.status as u16);
        for (name, value) in response.headers.iter() {
            builder = builder.header(name, value);
        }

        match response.body {
            ResponseBody::Bytes(b) => builder.body(Full::new(b).boxed()),
            ResponseBody::Stream(s) => builder.body(Arc::try_unwrap(s).unwrap()),
        }
    }
}
