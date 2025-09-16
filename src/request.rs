use std::sync::Arc;

use ahash::HashMap;
use http_body_util::{BodyExt, Full};
use pyo3::{
    exceptions::{PyAttributeError, PyException},
    prelude::*,
    types::PyDict,
};
use pyo3_stub_gen::derive::*;

use hyper::Uri;
use multer::bytes::Bytes;
use url::form_urlencoded;

use crate::multipart::parse_multipart;
use crate::response::Response;
use crate::routing::MatchRoute;
use crate::status::Status;
use crate::{
    json,
    multipart::File,
    session::{Session, SessionStore},
    templating::Template,
    IntoPyException, ProcessRequest, RequestContext,
};

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
/// # Request objects are typically created by the framework and
/// # passed to your handler functions:
///
/// @router.get("/hello")
/// def handler(request):
///     user_agent = request.headers.get("user-agent")
///     return f"Hello from {user_agent}"
/// ```
#[derive(Clone, Debug, Default)]
#[gen_stub_pyclass]
#[pyclass]
pub struct Request {
    /// The HTTP method of the request (e.g., GET, POST, PUT).
    #[pyo3(get)]
    pub method: String,
    /// The full URI of the request including path and query string.
    #[pyo3(get)]
    pub uri: String,
    /// HTTP headers as key-value pairs.
    #[pyo3(get)]
    pub headers: HashMap<String, String>,
    /// The raw data content of the request as a string, if present.
    #[pyo3(get)]
    pub data: Option<String>,
    /// Form data parsed from the request body, available when content type is application/x-www-form-urlencoded.
    #[pyo3(get)]
    pub form: Option<HashMap<String, String>>,
    /// Files uploaded in a multipart form request, mapping field names to File objects.
    #[pyo3(get)]
    pub files: Option<HashMap<String, File>>,
    pub app_data: Option<Arc<Py<PyAny>>>,
    pub template: Option<Arc<Template>>,
    pub ext: HashMap<String, Arc<PyObject>>,
    pub session: Option<Arc<Session>>,
    pub session_store: Option<Arc<SessionStore>>,
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
    pub fn new(method: String, uri: String, headers: HashMap<String, String>) -> Self {
        Self {
            method,
            uri,
            headers,
            ..Default::default()
        }
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
    /// @router.post("/api/data")
    /// def handle_data(request):
    ///     data = request.json()
    ///     value = data["key"]
    ///     return {"received": value}
    /// ```
    pub fn json(&self) -> PyResult<Py<PyDict>> {
        let data = self
            .data
            .as_ref()
            .ok_or_else(|| PyException::new_err("The body is not present"))?;
        json::loads(data)
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
    /// @router.get("/counter")
    /// def get_counter(request):
    ///     app_state = request.app_data
    ///     app_state.counter += 1
    ///     return {"count": app_state.counter}
    /// ```
    #[getter]
    fn app_data(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.app_data.as_ref().map(|d| d.clone_ref(py))
    }

    /// Parse and return the query parameters from the request URI.
    ///
    /// Args:
    ///     None
    ///
    /// Returns:
    ///     dict or None: Dictionary of query parameters, or None if no query string exists
    ///
    /// Raises:
    ///     Exception: If the URI cannot be parsed
    ///
    /// Example:
    /// ```python
    /// # For a request to /api?name=John&age=30
    /// @router.get("/api")
    /// def api_handler(request):
    ///     query = request.query()
    ///     name = query.get("name")
    ///     age = query.get("age")
    ///     return {"name": name, "age": age}
    /// ```
    fn query(&self) -> PyResult<Option<std::collections::HashMap<String, String>>> {
        let uri: Uri = self.uri.parse().into_py_exception()?;
        if let Some(query_string) = uri.query() {
            let parsed_query = form_urlencoded::parse(query_string.as_bytes())
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect();
            return Ok(Some(parsed_query));
        }
        Ok(None)
    }

    /// Get the session object for the current request.
    ///
    /// Use this to access or modify session data that persists across requests.
    ///
    /// Args:
    ///     None
    ///
    /// Returns:
    ///     Session: The session instance for this request
    ///
    /// Raises:
    ///     AttributeError: If session store is not configured on the server
    ///
    /// Example:
    /// ```python
    /// @router.get("/login")
    /// def login(request):
    ///     session = request.session()
    ///     session["user_id"] = 123
    ///     session["is_authenticated"] = True
    ///     return "Logged in successfully"
    /// ```
    pub fn session(&self) -> PyResult<Session> {
        let message = "Session not available. Make sure you've configured SessionStore.";
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| PyAttributeError::new_err(message))?;
        Ok(session.as_ref().clone())
    }

    fn get_cookie(&self, name: &str) -> Option<&str> {
        let cookie = self.headers.get("cookie")?;
        let cookies = cookie.split(';');
        for c in cookies {
            let (k, v) = c.split_once('=')?;
            if k == name {
                return Some(v);
            }
        }
        None
    }

    fn __getattr__(&self, py: Python<'_>, name: &str) -> PyResult<PyObject> {
        let message = format!("Request object has no attribute {name}");
        let obj = self
            .ext
            .get(name)
            .ok_or_else(|| PyAttributeError::new_err(message))?;
        Ok(obj.clone_ref(py))
    }

    fn __setattr__(&mut self, name: &str, value: PyObject) -> PyResult<()> {
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
    pub(crate) async fn handle(
        self,
        RequestContext {
            request_sender,
            routers,
            channel_capacity,
            cors,
            catchers,
        }: RequestContext,
    ) -> Result<hyper::Response<Full<Bytes>>, hyper::http::Error> {
        for router in routers {
            if let Some(match_route) = router.find(&self.method, &self.uri) {
                let (tx, mut rx) = tokio::sync::mpsc::channel(channel_capacity);
                let transmutate_route: MatchRoute = unsafe { std::mem::transmute(match_route) };

                let process_request = ProcessRequest {
                    tx,
                    cors: cors.clone(),
                    catchers: catchers.clone(),
                    router: Some(router),
                    match_route: Some(transmutate_route),
                    request: Arc::new(self.clone()),
                };

                if request_sender.send(process_request).await.is_ok() {
                    if let Some(response) = rx.recv().await {
                        return response.try_into();
                    }
                }
            }
        }

        let (tx, mut rx) = tokio::sync::mpsc::channel(channel_capacity);

        let process_request = ProcessRequest {
            tx,
            cors,
            catchers,
            router: None,
            match_route: None,
            request: Arc::new(self),
        };

        if request_sender.send(process_request).await.is_ok() {
            if let Some(response) = rx.recv().await {
                return response.try_into();
            }
        }

        let response: Response = Status::NOT_FOUND.into();
        response.try_into()
    }
}

pub struct RequestBuilder {
    method: String,
    uri: String,
    headers: HashMap<String, String>,
    app_data: Option<Arc<Py<PyAny>>>,
    template: Option<Arc<Template>>,
    session_store: Option<Arc<SessionStore>>,
    req: hyper::Request<hyper::body::Incoming>,
}

impl RequestBuilder {
    pub fn new(req: hyper::Request<hyper::body::Incoming>) -> Self {
        Self {
            method: req.method().to_string(),
            uri: req.uri().to_string(),
            headers: req
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
                .collect(),
            req,
            app_data: None,
            template: None,
            session_store: None,
        }
    }

    pub fn with_app_data(mut self, app_data: Option<Arc<Py<PyAny>>>) -> Self {
        self.app_data = app_data;
        self
    }

    pub fn with_template(mut self, template: Option<Arc<Template>>) -> Self {
        self.template = template;
        self
    }

    pub fn with_session_store(mut self, session_store: Option<Arc<SessionStore>>) -> Self {
        self.session_store = session_store;
        self
    }

    pub async fn build(self) -> PyResult<Request> {
        let mut request = Request::new(self.method, self.uri, self.headers);

        let bytes = self.req.collect().await.into_py_exception()?.to_bytes();
        let body = String::from_utf8_lossy(&bytes).to_string();

        if !body.is_empty() {
            request.data = Some(body.clone());
        }

        if let Some(content_type) = request.headers.get("content-type") {
            if content_type.starts_with("multipart/form-data") {
                let parsed_multipart = parse_multipart(content_type, bytes)
                    .await
                    .into_py_exception()?;
                request.form = Some(parsed_multipart.fields);
                request.files = Some(parsed_multipart.files);
            }
        }

        if let Some(store) = self.session_store {
            if let Some(session_id) =
                get_session_id(request.headers.get("cookie"), &store.cookie_name)
            {
                if let Ok(session) = store.get_session(Some(session_id)) {
                    request.session = Some(Arc::new(session));
                    request.session_store = Some(store.clone());
                }
            }
        }

        request.app_data = self.app_data;
        request.template = self.template;

        Ok(request)
    }
}

fn get_session_id(cookie_header: Option<&String>, cookie_name: &str) -> Option<String> {
    cookie_header.and_then(|cookies| {
        cookies
            .split(';')
            .filter_map(|cookie| {
                let cookie = cookie.trim();
                let mut parts = cookie.splitn(2, '=');
                if let (Some(name), Some(value)) = (
                    parts.next().map(|s| s.trim()),
                    parts.next().map(|s| s.trim()),
                ) {
                    if name == cookie_name {
                        return Some(value.to_string());
                    }
                }
                None
            })
            .next()
    })
}
