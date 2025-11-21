use std::{mem::transmute, sync::Arc};

use ahash::HashMap;
use pyo3::{ffi::c_str, prelude::*, types::PyDict, Py, PyAny};
use pyo3_stub_gen::derive::*;

use crate::{middleware::Middleware, IntoPyException};

pub type MatchRoute<'l> = matchit::Match<'l, 'l, &'l Route>;

/// A route definition that maps a URL path to a handler function.
///
/// Args:
///     path (str): The URL path pattern.
///     method (str, optional): The HTTP method (defaults to "GET").
///
/// Returns:
///     Route: A route object that can be registered with a router.
///
/// Example:
/// ```python
/// from oxapy import Route
///
/// def handler(request):
///     return "Hello, World!"
///
/// route = Route("/hello", "GET")
/// route = route(handler)  # Attach the handler
/// ```
#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone, Debug)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub handler: Arc<Py<PyAny>>,
}

impl Default for Route {
    fn default() -> Self {
        Python::attach(|py| Self {
            method: "GET".to_string(),
            path: String::default(),
            handler: Arc::new(py.None()),
        })
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl Route {
    #[new]
    #[pyo3(signature=(path, method=None))]
    pub fn new(path: String, method: Option<String>) -> Self {
        Route {
            method: method.unwrap_or("GET".to_string()),
            path,
            ..Default::default()
        }
    }

    fn __call__(&self, handler: Py<PyAny>) -> PyResult<Self> {
        Ok(Self {
            handler: Arc::new(handler),
            ..self.clone()
        })
    }

    fn __repr__(&self) -> String {
        format!("{:#?}", self)
    }
}

macro_rules! method_decorator {
    (
        $(
             $(#[$docs:meta])*
             $method:ident;
        )*
    ) => {
        $(
            $(#[$docs])*
            #[gen_stub_pyfunction]
            #[pyfunction]
            #[pyo3(signature = (path, handler = None))]
            pub fn $method(path: String, handler: Option<Py<PyAny>>, py: Python<'_>) -> Route {
                Route {
                    method: stringify!($method).to_string().to_uppercase(),
                    path,
                    handler: Arc::new(handler.unwrap_or(py.None()))
                }
            }
        )+
    };
}

method_decorator!(
    /// Registers an HTTP GET route.
    ///
    /// Parameters:
    ///     path (str): The route path, which may include parameters (e.g. `/items/{id}`).
    ///     handler (callable | None): Optional Python function that handles the request.
    ///
    /// Returns:
    ///     Route: A GET Route instance.
    ///
    /// Example:
    /// ```python
    /// get("/hello/{name}", lambda req, name: f"Hello, {name}!")
    /// ```
    get;

    /// Registers an HTTP POST route.
    ///
    /// Parameters:
    ///     path (str): The POST route path.
    ///     handler (callable | None): Optional Python function that handles the request.
    ///
    /// Returns:
    ///     Route: A POST Route instance.
    ///
    /// Example:
    /// ```python
    /// post("/users", lambda req: {"id": 1, "name": req.json()["name"]})
    /// ```
    post;

    /// Registers an HTTP DELETE route.
    ///
    /// Parameters:
    ///     path (str): The DELETE route path.
    ///     handler (callable | None): Optional Python function that handles the request.
    ///
    /// Returns:
    ///     Route: A DELETE Route instance.
    ///
    /// Example:
    /// ```python
    /// delete("/items/{id}", lambda req, id: f"Deleted {id}")
    /// ```
    delete;

    /// Registers an HTTP PATCH route.
    ///
    /// Parameters:
    ///     path (str): The PATCH route path.
    ///     handler (callable | None): Optional Python function for partial updates.
    ///
    /// Returns:
    ///     Route: A PATCH Route instance.
    ///
    /// Example:
    /// ```python
    /// patch("/users/{id}", lambda req, id: req.json())
    /// ```
    patch;

    /// Registers an HTTP PUT route.
    ///
    /// Parameters:
    ///     path (str): The PUT route path.
    ///     handler (callable | None): Optional Python function for full replacement.
    ///
    /// Returns:
    ///     Route: A PUT Route instance.
    ///
    /// Example:
    /// ```python
    /// put("/users/{id}", lambda req, id: req.json())
    /// ```
    put;

    /// Registers an HTTP HEAD route.
    ///
    /// Parameters:
    ///     path (str): The HEAD route path.
    ///     handler (callable | None): Optional function for returning headers only.
    ///
    /// Returns:
    ///     Route: A HEAD Route instance.
    ///
    /// Example:
    /// ```python
    /// head("/status", lambda req: None)
    /// ```
    head;

    /// Registers an HTTP OPTIONS route.
    ///
    /// Parameters:
    ///     path (str): The OPTIONS route path.
    ///     handler (callable | None): Optional handler that returns allowed methods.
    ///
    /// Returns:
    ///     Route: An OPTIONS Route instance.
    ///
    /// Example:
    /// ```python
    /// options("/users", lambda req: {"Allow": "GET, POST"})
    /// ```
    options;
);

/// A router for handling HTTP routes.
///
/// The Router is responsible for registering routes and handling HTTP requests.
/// It supports path parameters, middleware, and different HTTP methods.
///
/// A `base_path` can be provided to prepend a path to all routes.
///
/// Returns:
///     Router: A new router instance.
///
/// Example:
/// ```python
/// from oxapy import Router
///
/// # Router with a base path
/// router = Router("/api/v1")
///
/// @router.get("/hello/{name}")
/// def hello(request, name):
///     return f"Hello, {name}!"
///
/// # The route will be /api/v1/hello/{name}
/// ```
#[gen_stub_pyclass]
#[pyclass]
#[derive(Default, Clone, Debug)]
pub struct Router {
    pub base_path: Option<String>,
    pub routes: HashMap<String, matchit::Router<Route>>,
    pub middlewares: Vec<Middleware>,
    pub services: Vec<Arc<Router>>,
}

#[gen_stub_pymethods]
#[pymethods]
impl Router {
    /// Create a new Router instance.
    ///
    /// Returns:
    ///     Router: A new router with no routes or middleware.
    ///
    /// Example:
    /// ```python
    /// router = Router()
    /// ```
    #[new]
    #[pyo3(signature=(base_path = None))]
    pub fn new(base_path: Option<String>) -> Self {
        Router {
            base_path,
            ..Default::default()
        }
    }

    /// Add middleware to the router.
    ///
    /// Middleware functions are executed in the order they are added,
    /// before the route handler.
    ///
    /// Args:
    ///     middleware (callable): A function that will process requests before route handlers.
    ///
    /// Returns:
    ///     None
    ///
    /// Example:
    /// ```python
    /// def auth_middleware(request, next, **kwargs):
    ///     if "authorization" not in request.headers:
    ///         return Status.UNAUTHORIZED
    ///     return next(request, **kwargs)
    ///
    /// router.middleware(auth_middleware)
    /// ```
    fn middleware(&mut self, middleware: Py<PyAny>) -> Self {
        let middleware = Middleware::new(middleware);
        self.middlewares.push(middleware);
        self.clone()
    }

    /// Register a route with the router.
    ///
    /// Args:
    ///     route (Route): The route to register.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     Exception: If the route cannot be added.
    ///
    /// Example:
    /// ```python
    /// from oxapy import get
    ///
    /// def hello_handler(request):
    ///     return "Hello World!"
    ///
    /// route = get("/hello", hello_handler)
    /// router.route(route)
    /// ```
    fn route(&mut self, route: &Route) -> PyResult<Self> {
        let method_router = self.routes.entry(route.method.clone()).or_default();
        let full_path = match self.base_path {
            Some(ref base_path) => {
                let combined = format!("{base_path}/{}", route.path);
                let segments: Vec<&str> = combined.split("/").filter(|s| !s.is_empty()).collect();
                format!("/{}", segments.join("/"))
            }
            None => route.path.clone(),
        };
        method_router
            .insert(full_path, route.clone())
            .into_py_exception()?;
        Ok(self.clone())
    }

    /// Register multiple routes with the router.
    ///
    /// Args:
    ///     routes (list): A list of Route objects to register.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     Exception: If any route cannot be added.
    ///
    /// Example:
    /// ```python
    /// from oxapy import get, post
    ///
    /// def hello_handler(request):
    ///     return "Hello World!"
    ///
    /// def submit_handler(request):
    ///     return "Form submitted!"
    ///
    /// routes = [
    ///     get("/hello", hello_handler),
    ///     post("/submit", submit_handler)
    /// ]
    /// router.routes(routes)
    /// ```
    fn routes(&mut self, routes: Vec<Route>) -> PyResult<Self> {
        for ref route in routes {
            self.route(route)?;
        }
        Ok(self.clone())
    }

    fn service(&mut self) -> Self {
        self.services.push(Arc::new(self.clone()));
        self.clone()
    }

    fn __repr__(&self) -> String {
        format!("{:#?}", self)
    }
}

impl Router {
    pub(crate) fn find<'l>(&'l self, method: &str, uri: &'l str) -> Option<MatchRoute<'l>> {
        let path = uri.split('?').next().unwrap_or(uri);
        let router = self.routes.get(method)?;
        let route = router.at(path).ok()?;
        let route: MatchRoute = unsafe { transmute(route) };
        Some(route)
    }
}

/// Create a route for serving static files.
///
/// Args:
///     directory (str): The directory containing static files.
///     path (str): The URL path at which to serve the files.
///
/// Returns:
///     Route: A route configured to serve static files.
///
/// Example:
/// ```python
/// from oxapy import Router, static_file
///
/// router = Router()
/// router.route(static_file("/static", "./static"))
/// # This will serve files from ./static directory at /static URL path
/// ```
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature=(path="/static", directory="./static"))]
pub fn static_file(path: &str, directory: &str, py: Python<'_>) -> PyResult<Route> {
    let pathlib = py.import("pathlib")?;
    let oxapy = py.import("oxapy")?;
    let mimetypes = py.import("mimetypes")?;

    let globals = &PyDict::new(py);
    globals.set_item("Path", pathlib.getattr("Path")?)?;
    globals.set_item("directory", directory)?;
    globals.set_item("Status", oxapy.getattr("Status")?)?;
    globals.set_item("Response", oxapy.getattr("Response")?)?;
    globals.set_item("mimetypes", mimetypes)?;

    py.run(
        c_str!(
            r#"
def static_file(request, path):
    file_path = f"{directory}/{path}"
    try:
        with open(file_path, "rb") as f: content = f.read()
        content_type, _ = mimetypes.guess_type(file_path)
        return Response(content, content_type = content_type or "application/octet-stream")
    except FileNotFoundError:
        return Response("File not found", Status.NOT_FOUND)
"#
        ),
        Some(globals),
        None,
    )?;

    let handler = globals.get_item("static_file")?.unwrap();

    let route = Route {
        path: format!("/{path}/{{*path}}"),
        handler: Arc::new(handler.into()),
        ..Default::default()
    };

    Ok(route)
}
