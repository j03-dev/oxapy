use std::sync::Arc;
use tokio::sync::oneshot;

use ahash::{HashMap, HashMapExt};
use http_body_util::BodyExt;
use hyper::header::{CONTENT_TYPE, COOKIE, HeaderMap, HeaderName, HeaderValue};
use hyper::{Method, Uri};
use pyo3::{
    exceptions::{PyAttributeError, PyException},
    prelude::*,
    types::PyDict,
};
use pyo3_stub_gen::derive::*;
use url::form_urlencoded;

use crate::response::Response;
use crate::status::Status;
use crate::{
    Context, IntoPyException, ProcessRequest, json, multipart::File, templating::Template,
};
use crate::{middleware::Middleware, routing::OwnedMatchRoute};
use crate::{multipart::parse_multipart, response::Body};

/// HTTP request object containing information about the incoming request.
///
/// This class provides access to request details such as method, URI, headers,
/// body content, form data, uploaded files, and session information.
///
/// Args:
///     method (str): The HTTP method of the request (GET, POST, etc.)
///     uri (str): The URI of the request
///     headers (dict): HTTP headers as key-value pairs
///
/// Returns:
///     Request: A new request object
///
/// Example:
/// ```python
/// from oxapy import get
///
/// # Request objects are typically created by the framework and
/// # passed to your handler functions:
///
/// @get("/hello")
/// def handler(request):
///     user_agent = request.headers.get("user-agent")
///     return f"Hello from {user_agent}"
/// ```
#[gen_stub_pyclass]
#[pyclass(from_py_object)]
#[derive(Clone, Debug, Default)]
pub struct Request {
    /// The HTTP method of the request (e.g., GET, POST, PUT).
    pub method: Method,
    /// The full URI of the request including path and query string.
    pub uri: Uri,
    /// HTTP headers as key-value pairs.
    pub headers: HeaderMap,
    /// The raw data content of the request as a string, if present.
    #[pyo3(get)]
    pub data: Option<String>,
    /// Form data parsed from the request body, available when content type is application/x-www-form-urlencoded.
    #[pyo3(get)]
    pub form: HashMap<String, String>,
    /// Files uploaded in a multipart form request, mapping field names to File objects.
    #[pyo3(get)]
    pub files: HashMap<String, File>,
    pub app_data: Option<Arc<Py<PyAny>>>,
    pub template: Option<Arc<Template>>,
    pub ext: HashMap<String, Arc<Py<PyAny>>>,
}

#[gen_stub_pymethods]
#[pymethods]
impl Request {
    /// Create a new Request instance.
    ///
    /// Note: This is primarily for internal use. Request objects are typically created
    /// by the framework and passed to your handler functions.
    ///
    /// Args:
    ///     method (str): The HTTP method of the request (GET, POST, etc.)
    ///     uri (str): The URI of the request
    ///     headers (dict): HTTP headers as key-value pairs
    ///
    /// Returns:
    ///     Request: A new request object
    #[new]
    #[gen_stub(override_return_type(type_repr = "typing_extensions.Self", imports = ("typing_extensions",)))]
    pub fn new(method: String, uri: String, headers: HashMap<String, String>) -> Self {
        let method = method.parse::<Method>().unwrap_or(Method::GET);
        let uri = uri.parse::<Uri>().unwrap_or_default();
        let mut header_map = HeaderMap::with_capacity(headers.len());
        for (name, value) in headers {
            if let (Ok(name), Ok(value)) =
                (HeaderName::try_from(name), HeaderValue::from_str(&value))
            {
                header_map.append(name, value);
            }
        }
        Self {
            method,
            uri,
            headers: header_map,
            ..Default::default()
        }
    }

    #[getter]
    fn method(&self) -> String {
        self.method.as_str().to_string()
    }

    #[getter]
    fn uri(&self) -> String {
        self.uri.to_string()
    }

    #[getter]
    fn headers(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        for (name, value) in self.headers.iter() {
            if let Ok(value) = value.to_str() {
                dict.set_item(name.as_str(), value)?;
            }
        }
        Ok(dict.into())
    }

    /// Parse the request body as JSON and return it as a dictionary.
    ///
    /// Args:
    ///     None
    ///
    /// Returns:
    ///     dict: The parsed JSON data as a Python dictionary
    ///
    /// Raises:
    ///     Exception: If the body is not present or cannot be parsed as JSON
    ///
    /// Example:
    /// ```python
    /// from oxapy import post
    ///
    /// @post("/api/data")
    /// def handle_data(request):
    ///     data = request.json()
    ///     value = data["key"]
    ///     return {"received": value}
    /// ```
    pub fn json(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let data = self
            .data
            .as_ref()
            .ok_or_else(|| PyException::new_err("The body is not present"))?;
        json::loads(data, py)
    }

    /// Get application-wide data that was set with HttpServer.app_data.
    ///
    /// Args:
    ///     None
    ///
    /// Returns:
    ///     any: The application data object, or None if no app_data was set
    ///
    /// Example:
    /// ```python
    /// from oxapy import get
    ///
    /// @get("/counter")
    /// def get_counter(request):
    ///     app_state = request.app_data
    ///     app_state.counter += 1
    ///     return {"count": app_state.counter}
    /// ```
    #[getter]
    fn app_data(&self, py: Python<'_>) -> Py<PyAny> {
        self.app_data
            .as_ref()
            .map(|d| d.clone_ref(py))
            .unwrap_or(py.None())
    }

    /// Parse and return the query parameters from the request URI.
    ///
    /// Args:
    ///     None
    ///
    /// Returns:
    ///     dict: Dictionary of query parameters
    ///
    /// Raises:
    ///     Exception: If the URI cannot be parsed
    ///
    /// Example:
    /// ```python
    /// from oxapy import get
    ///
    /// # For a request to /api?name=John&age=30
    /// @get("/api")
    /// def api_handler(request):
    ///     query = request.query
    ///     name = query.get("name")
    ///     age = query.get("age")
    ///     return {"name": name, "age": age}
    /// ```
    #[getter]
    fn query(&self) -> HashMap<String, String> {
        match self.uri.query() {
            Some(query_string) => form_urlencoded::parse(query_string.as_bytes())
                .map(|(key, value)| (key.into_owned(), value.into_owned()))
                .collect(),
            None => HashMap::new(),
        }
    }

    /// Get cookie value by the name from the request headers
    ///
    /// Retrieves the value of a specific cookie from the HTTP Cookie header.
    /// Returns None if the cookie name is not found or if no Cookie header exists.
    ///
    /// Args:
    ///     name (str): The name of the cookie to retrieve
    ///
    /// Returns:
    ///     (str, optional): cookie's value and return none if name is presente
    ///
    /// Example
    /// ```python
    /// from oxapy import get, render
    ///
    /// @get("/")
    /// def index(request):
    ///     theme = request.get_cookie("theme") or "light"
    ///     render(request, "index.html.j2", {"theme": theme})
    /// ```
    fn get_cookie(&self, name: &str) -> Option<&str> {
        let cookie = self.headers.get(COOKIE)?;
        let cookie = cookie.to_str().ok()?;
        let cookies = cookie.split(';');
        for c in cookies {
            let (k, v) = c.trim().split_once('=')?;
            if k == name {
                return Some(v);
            }
        }
        None
    }

    fn __getattr__(&self, py: Python<'_>, name: &str) -> PyResult<Py<PyAny>> {
        let message = format!("Request object has no attribute {name}");
        let obj = self
            .ext
            .get(name)
            .ok_or_else(|| PyAttributeError::new_err(message))?;
        Ok(obj.clone_ref(py))
    }

    fn __setattr__(&mut self, name: &str, value: Py<PyAny>) -> PyResult<()> {
        match name {
            "method" | "uri" | "headers" | "body" | "template" => Err(PyException::new_err(
                format!("Attribute '{}' is read-only and cannot be set", name),
            )),
            _ => {
                self.ext.insert(name.to_string(), Arc::new(value));
                Ok(())
            }
        }
    }

    pub fn __repr__(&self) -> String {
        format!("{:#?}", self)
    }
}

impl Request {
    pub(crate) async fn process(
        self,
        ctx: Arc<Context>,
    ) -> Result<hyper::Response<Body>, hyper::http::Error> {
        if self.method == Method::OPTIONS
            && let Some(ref cors) = ctx.cors
        {
            return Response::try_from((**cors).clone())
                .unwrap_or_else(Response::from)
                .try_into();
        }

        let matched = ctx.routers.iter().find_map(|router| {
            router
                .find(self.method.as_str(), self.uri.path())
                .map(|m| (OwnedMatchRoute::from(m), router.middlewares.clone()))
        });

        if let Some((match_route, middlewares)) = matched {
            self.handle_found_route(&ctx, match_route, middlewares)
                .await
        } else {
            self.handle_not_found(&ctx).await
        }
    }

    async fn handle_found_route(
        self,
        ctx: &Context,
        match_route: OwnedMatchRoute,
        middlewares: Option<Arc<[Middleware]>>,
    ) -> Result<hyper::Response<Body>, hyper::http::Error> {
        let (response_sender, response_receiver) = oneshot::channel();

        let process_request = ProcessRequest {
            match_route: Some(match_route),
            middlewares,
            request: self,
            response_sender,
            wrapper: ctx.wrapper.clone(),
            cors: ctx.cors.clone(),
        };

        Self::send_and_wait_response(&ctx.request_sender, process_request, response_receiver).await
    }

    async fn handle_not_found(
        self,
        ctx: &Context,
    ) -> Result<hyper::Response<Body>, hyper::http::Error> {
        let (response_sender, response_receiver) = oneshot::channel();

        let process_request = ProcessRequest {
            match_route: None,
            middlewares: None,
            request: self,
            response_sender,
            wrapper: ctx.wrapper.clone(),
            cors: ctx.cors.clone(),
        };

        Self::send_and_wait_response(&ctx.request_sender, process_request, response_receiver).await
    }

    async fn send_and_wait_response<T>(
        request_sender: &tokio::sync::mpsc::Sender<T>,
        process_request: T,
        rx: oneshot::Receiver<Response>,
    ) -> Result<hyper::Response<Body>, hyper::http::Error> {
        if request_sender.send(process_request).await.is_ok()
            && let Ok(response) = rx.await
        {
            return response.try_into();
        }
        Response::from(Status::INTERNAL_SERVER_ERROR).try_into()
    }
}

pub struct RequestBuilder {
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    app_data: Option<Arc<Py<PyAny>>>,
    template: Option<Arc<Template>>,
    body: hyper::body::Incoming,
}

impl RequestBuilder {
    pub fn new(req: hyper::Request<hyper::body::Incoming>) -> Self {
        let (parts, body) = req.into_parts();

        Self {
            method: parts.method,
            uri: parts.uri,
            headers: parts.headers,
            body,
            app_data: None,
            template: None,
        }
    }

    pub fn with_app_data(mut self, app_data: &Option<Arc<Py<PyAny>>>) -> Self {
        self.app_data = app_data.clone();
        self
    }

    pub fn with_template(mut self, template: &Option<Arc<Template>>) -> Self {
        self.template = template.clone();
        self
    }

    pub async fn build(self) -> PyResult<Request> {
        let mut request = Request {
            method: self.method,
            uri: self.uri,
            headers: self.headers,
            ..Default::default()
        };

        let bytes = self.body.collect().await.into_py_exception()?.to_bytes();

        if let Some(content_type) = request
            .headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
        {
            if content_type.starts_with("multipart/form-data") {
                let parsed_multipart = parse_multipart(content_type, bytes)
                    .await
                    .into_py_exception()?;
                request.form = parsed_multipart.fields;
                request.files = parsed_multipart.files;
            } else if content_type.starts_with("application/json") {
                if !bytes.is_empty() {
                    request.data = Some(match String::from_utf8(bytes.to_vec()) {
                        Ok(body) => body,
                        Err(err) => String::from_utf8_lossy(err.as_bytes()).into_owned(),
                    });
                }
            } else if !bytes.is_empty() {
                request.form = form_urlencoded::parse(bytes.as_ref())
                    .map(|(k, v)| (k.into_owned(), v.into_owned()))
                    .collect();
            }
        }

        request.app_data = self.app_data;
        request.template = self.template;

        Ok(request)
    }
}
