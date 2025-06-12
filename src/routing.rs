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
/// ```python
/// from oxapy import Route
///
/// def handler(request):
///     return "Hello, World!"
///
/// route = Route("/hello", "GET")
/// route = route(handler)  # Attach the handler
/// ```
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
    (
        $(
             $(#[$docs:meta])*
             $method:ident;
        )*
    ) => {
        $(
            $(#[$docs])*
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
/// ```python
/// from oxapy import Router, get
///
/// router = Router()
///
/// @router.get("/hello/{name}")
/// def hello(request, name):
///     return f"Hello, {name}!"
/// ```
#[derive(Default, Clone, Debug)]
#[pyclass]
pub struct Router {
    pub routes: Arc<RwLock<HashMap<String, matchit::Router<Route>>>>,
    pub middlewares: Vec<Middleware>,
}

macro_rules! impl_router {
    (
        $(
             $(#[$docs:meta])*
             $method:ident;
        )*
    ) => {
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
            /// ```python
            /// def auth_middleware(request, next, **kwargs):
            ///     if "authorization" not in request.headers:
            ///         return Status.UNAUTHORIZED
            ///     return next(request, **kwargs)
            ///
            /// router.middleware(auth_middleware)
            /// ```
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
            /// ```python
            /// from oxapy import get
            ///
            /// def hello_handler(request):
            ///     return "Hello World!"
            ///
            /// route = get("/hello", hello_handler)
            /// router.route(route)
            /// ```
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
            fn routes(&mut self, routes: Vec<Route>) -> PyResult<()> {
                for ref route in routes {
                    self.route(route)?;
                }
                Ok(())
            }

            $(
                $(#[$docs])*
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

impl_router!(
    /// Register a GET route using the decorator `@router.get(path)`.
    ///
    /// Example:
    /// ```python
    /// @router.get("/hello")
    /// def hello(request):
    ///     return "Hello, world!"
    /// ```
    get;

    /// Register a POST route using the decorator `@router.post(path)`.
    ///
    /// Example:
    /// ```python
    /// @router.post("/submit")
    /// def submit(request):
    ///     return "Submitted!"
    /// ```
    post;

    /// Register a PUT route using the decorator `@router.put(path)`.
    ///
    /// Example:
    /// ```python
    /// @router.put("/items/{id}")
    /// def update_item(request, id):
    ///     return f"Updated item {id}"
    /// ```
    put;

    /// Register a PATCH route using the decorator `@router.patch(path)`.
    ///
    /// Example:
    /// ```python
    /// @router.patch("/items/{id}")
    /// def patch_item(request, id):
    ///     return f"Patched item {id}"
    /// ```
    patch;

    /// Register a DELETE route using the decorator `@router.delete(path)`.
    ///
    /// Example:
    /// ```python
    /// @router.delete("/items/{id}")
    /// def delete_item(request, id):
    ///     return f"Deleted item {id}"
    /// ```
    delete;

    /// Register a HEAD route using the decorator `@router.head(path)`.
    ///
    /// Example:
    /// ```python
    /// @router.head("/ping")
    /// def head_ping(request):
    ///     return ""
    /// ```
    head;

    /// Register an OPTIONS route using the decorator `@router.options(path)`.
    ///
    /// Example:
    /// ```python
    /// @router.options("/data")
    /// def options_data(request):
    ///     return "OPTIONS OK"
    /// ```
    options;
);

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
/// ```python
/// from oxapy import Router, static_file
///
/// router = Router()
/// router.route(static_file("./static", "static"))
/// # This will serve files from ./static directory at /static URL path
/// ```
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
