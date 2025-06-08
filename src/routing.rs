use std::{
    mem::transmute,
    sync::{Arc, RwLock},
};

use ahash::HashMap;
use pyo3::{ffi::c_str, prelude::*, types::PyDict, Py, PyAny};

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
///     ```python
///     from oxapy import Route
///
///     def handler(request):
///         return "Hello, World!"
///
///     route = Route("/hello", "GET")
///     route = route(handler)  # Attach the handler
///     ```
#[derive(Clone, Debug)]
#[pyclass]
pub struct Route {
    pub method: String,
    pub path: String,
    pub handler: Arc<Py<PyAny>>,
}

impl Default for Route {
    fn default() -> Self {
        Python::with_gil(|py| Self {
            method: "GET".to_string(),
            path: String::default(),
            handler: Arc::new(py.None()),
        })
    }
}

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
    ($($method:ident),*) => {
        $(
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

method_decorator!(get, post, put, patch, delete, head, options);

#[derive(Clone)]
#[pyclass]
struct RouteBuilder {
    method: String,
    router: Router,
    path: String,
}

#[pymethods]
impl RouteBuilder {
    fn __call__(&mut self, handler: Py<PyAny>) -> PyResult<Route> {
        let route = Route {
            method: self.method.clone(),
            path: self.path.clone(),
            handler: Arc::new(handler),
        };

        self.router.route(&route)?;

        Ok(route)
    }
}

/// A router for handling HTTP routes.
///
/// The Router is responsible for registering routes and handling HTTP requests.
/// It supports path parameters, middleware, and different HTTP methods.
///
/// Returns:
///     Router: A new router instance.
///
/// Example:
///     ```python
///     from oxapy import Router, get
///
///     router = Router()
///
///     @router.get("/hello/{name}")
///     def hello(request, name):
///         return f"Hello, {name}!"
///     ```
#[derive(Default, Clone, Debug)]
#[pyclass]
pub struct Router {
    pub routes: Arc<RwLock<HashMap<String, matchit::Router<Route>>>>,
    pub middlewares: Vec<Middleware>,
}

macro_rules! impl_router {
    ($($method:ident),*) => {
        #[pymethods]
        impl Router {
            /// Create a new Router instance.
            ///
            /// Returns:
            ///     Router: A new router with no routes or middleware.
            ///
            /// Example:
            ///     ```python
            ///     router = Router()
            ///     ```
            #[new]
            pub fn new() -> Self {
                Router::default()
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
            ///     ```python
            ///     def auth_middleware(request, next, **kwargs):
            ///         if "authorization" not in request.headers:
            ///             return Status.UNAUTHORIZED
            ///         return next(request, **kwargs)
            ///
            ///     router.middleware(auth_middleware)
            ///     ```
            fn middleware(&mut self, middleware: Py<PyAny>) {
                let middleware = Middleware::new(middleware);
                self.middlewares.push(middleware);
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
            ///     ```python
            ///     from oxapy import get
            ///
            ///     def hello_handler(request):
            ///         return "Hello World!"
            ///
            ///     route = get("/hello", hello_handler)
            ///     router.route(route)
            ///     ```
            fn route(&mut self, route: &Route) -> PyResult<()> {
                let mut ptr_mr = self.routes.write().unwrap();
                let method_router = ptr_mr.entry(route.method.clone()).or_default();
                method_router
                    .insert(&route.path, route.clone())
                    .into_py_exception()?;
                Ok(())
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
            ///     ```python
            ///     from oxapy import get, post
            ///
            ///     def hello_handler(request):
            ///         return "Hello World!"
            ///
            ///     def submit_handler(request):
            ///         return "Form submitted!"
            ///
            ///     routes = [
            ///         get("/hello", hello_handler),
            ///         post("/submit", submit_handler)
            ///     ]
            ///     router.routes(routes)
            ///     ```
            fn routes(&mut self, routes: Vec<Route>) -> PyResult<()> {
                for ref route in routes {
                    self.route(route)?;
                }
                Ok(())
            }

        $(
            fn $method(&self, path: String) -> PyResult<RouteBuilder> {
                Ok(RouteBuilder {
                    method: stringify!($method).to_string().to_uppercase(),
                    router: self.clone(),
                    path,
                })
            }
        )+

            fn __repr__(&self) -> String {
                format!("{:#?}", self)
            }
        }
    };
}

impl_router!(get, post, put, patch, delete, head, options);

impl Router {
    pub(crate) fn find<'l>(&'l self, method: &str, uri: &'l str) -> Option<MatchRoute<'l>> {
        let path = uri.split('?').next().unwrap_or(uri);
        let routes_guard = self.routes.read().ok()?;
        let router = routes_guard.get(method)?;
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
///     ```python
///     from oxapy import Router, static_file
///
///     router = Router()
///     router.route(static_file("./static", "static"))
///     # This will serve files from ./static directory at /static URL path
///     ```
#[pyfunction]
pub fn static_file(directory: String, path: String, py: Python<'_>) -> PyResult<Route> {
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
