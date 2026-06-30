use std::{mem::transmute, sync::Arc};

use ahash::HashMap;
use pyo3::{Py, PyAny, prelude::*};
use pyo3_stub_gen::derive::*;

use crate::{IntoPyException, middleware::Middleware};

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
#[pyclass(from_py_object)]
#[derive(Clone, Debug)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub handler: Arc<Py<PyAny>>,
    pub sequence: usize,
}

impl Default for Route {
    fn default() -> Self {
        Python::attach(|py| Self {
            method: "GET".to_string(),
            path: String::default(),
            handler: Arc::new(py.None()),
            sequence: 0,
        })
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl Route {
    #[new]
    #[pyo3(signature=(path, method=None))]
    #[gen_stub(override_return_type(type_repr = "typing_extensions.Self", imports = ("typing_extensions",)))]
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

macro_rules! methods {
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
                    sequence: 0,
                    handler: Arc::new(handler.unwrap_or(py.None()))
                }
            }
        )+
    };
}

methods!(
    /// Registers an HTTP GET route.
    ///
    /// Can be used as a decorator or as a function to create a `Route` object.
    /// When used as a decorator, the decorated function must be registered with a `Router`.
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
    /// from oxapy import Router, get
    ///
    /// router = Router()
    ///
    /// # As a function
    /// def get_items(request):
    ///     return {"items": []}
    /// router.route(get("/items", get_items))
    ///
    /// # As a decorator
    /// @get("/items/{item_id}")
    /// def get_item(request, item_id: int):
    ///     return {"item_id": item_id}
    ///
    /// router.route(get_item)
    /// ```
    get;

    /// Registers an HTTP POST route.
    ///
    /// Can be used as a decorator or as a function to create a `Route` object.
    /// When used as a decorator, the decorated function must be registered with a `Router`.
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
    /// from oxapy import Router, post
    ///
    /// router = Router()
    ///
    /// # As a function
    /// def create_user(request):
    ///     return request.json()
    /// router.route(post("/users", create_user))
    ///
    /// # As a decorator
    /// @post("/items")
    /// def create_item(request):
    ///     return {"status": "created"}
    ///
    /// router.route(create_item)
    /// ```
    post;

    /// Registers an HTTP DELETE route.
    ///
    /// Can be used as a decorator or as a function to create a `Route` object.
    /// When used as a decorator, the decorated function must be registered with a `Router`.
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
    /// from oxapy import Router, delete
    ///
    /// router = Router()
    ///
    /// # As a function
    /// router.route(delete("/items/{id}", lambda req, id: f"Deleted {id}"))
    ///
    /// # As a decorator
    /// @delete("/users/{user_id}")
    /// def delete_user(request, user_id: int):
    ///     return {"status": "deleted", "user_id": user_id}
    ///
    /// router.route(delete_user)
    /// ```
    delete;

    /// Registers an HTTP PATCH route.
    ///
    /// Can be used as a decorator or as a function to create a `Route` object.
    /// When used as a decorator, the decorated function must be registered with a `Router`.
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
    /// from oxapy import Router, patch
    ///
    /// router = Router()
    ///
    /// # As a function
    /// router.route(patch("/users/{id}", lambda req, id: req.json()))
    ///
    /// # As a decorator
    /// @patch("/items/{item_id}")
    /// def update_item_partial(request, item_id: int):
    ///     return {"status": "patched", "item_id": item_id}
    ///
    /// router.route(update_item_partial)
    /// ```
    patch;

    /// Registers an HTTP PUT route.
    ///
    /// Can be used as a decorator or as a function to create a `Route` object.
    /// When used as a decorator, the decorated function must be registered with a `Router`.
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
    /// from oxapy import Router, put
    ///
    /// router = Router()
    ///
    /// # As a function
    /// router.route(put("/users/{id}", lambda req, id: req.json()))
    ///
    /// # As a decorator
    /// @put("/items/{item_id}")
    /// def update_item_full(request, item_id: int):
    ///     return {"status": "updated", "item_id": item_id}
    ///
    /// router.route(update_item_full)
    /// ```
    put;

    /// Registers an HTTP HEAD route.
    ///
    /// Can be used as a decorator or as a function to create a `Route` object.
    /// When used as a decorator, the decorated function must be registered with a `Router`.
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
    /// from oxapy import Router, head, Response
    ///
    /// router = Router()
    ///
    /// # As a function
    /// router.route(head("/status", lambda req: Response("", status=200)))
    ///
    /// # As a decorator
    /// @head("/health")
    /// def health_check(request):
    ///     return Response("", status=200)
    ///
    /// router.route(health_check)
    /// ```
    head;

    /// Registers an HTTP OPTIONS route.
    ///
    /// Can be used as a decorator or as a function to create a `Route` object.
    /// When used as a decorator, the decorated function must be registered with a `Router`.
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
    /// from oxapy import Router, options
    ///
    /// router = Router()
    ///
    /// # As a function
    /// router.route(options("/users", lambda req: {"Allow": "GET, POST"}))
    ///
    /// # As a decorator
    /// @options("/items")
    /// def item_options(request):
    ///     return {"Allow": "GET, POST, PUT, DELETE"}
    ///
    /// router.route(item_options)
    /// ```
    options;
);

/// A router for handling HTTP routes.
///
/// The Router is responsible for registering routes and handling HTTP requests.
/// It supports path parameters, middleware, and different HTTP methods.
///
/// Middleware applies to all routes registered **after** it within the same router.
/// To isolate middleware to specific groups, create separate `Router` instances
/// and attach each one to the server.
///
/// A `base_path` can be provided to prepend a path to all routes.
///
/// Returns:
///     Router: A new router instance.
///
/// Example:
/// ```python
/// from oxapy import Router, get
///
/// # Router with a base path
/// router = Router("/api/v1")
///
/// @get("/hello/{name}")
/// def hello(request, name):
///     return f"Hello, {name}!"
///
/// router.route(hello)
///
/// # The route will be /api/v1/hello/{name}
/// ```
#[gen_stub_pyclass]
#[pyclass(from_py_object)]
#[derive(Default, Clone, Debug)]
pub struct Router {
    pub base_path: Option<String>,
    pub count: usize,
    pub middlewares: Vec<Middleware>,
    pub routes: HashMap<String, matchit::Router<Route>>,
}

impl Router {
    pub fn find<'l>(&'l self, method: &str, uri: &'l str) -> Option<MatchRoute<'l>> {
        let path = uri.split('?').next().unwrap_or(uri);
        let router = self.routes.get(method)?;
        let route = router.at(path).ok()?;
        let route: MatchRoute = unsafe { transmute(route) };
        Some(route)
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl Router {
    /// Create a new Router instance.
    ///
    /// Args:
    ///     base_path (str, optional): A base path to prepend to all routes registered with this router.
    ///
    /// Returns:
    ///     Router: A new router with no routes or middleware.
    ///
    /// Example:
    /// ```python
    /// # Router with a base path
    /// router = Router("/api/v1")
    ///
    /// # Router without a base path
    /// router = Router()
    ///
    /// # To isolate middleware groups, create separate routers:
    /// public_routes = Router()
    /// protected_routes = Router()
    /// server.attach(public_routes).attach(protected_routes)
    ///```
    #[new]
    #[pyo3(signature=(base_path = None))]
    #[gen_stub(override_return_type(type_repr = "typing_extensions.Self", imports = ("typing_extensions",)))]
    pub fn new(base_path: Option<String>) -> Self {
        Router {
            base_path,
            count: 0,
            middlewares: Vec::new(),
            routes: HashMap::default(),
        }
    }

    /// Add a middleware to the router.
    ///
    /// Middleware only applies to routes registered **after** it within the same router.
    /// This lets you layer middleware naturally by registration order.
    /// Use separate `Router` instances for full middleware isolation across groups.
    /// Middleware functions are executed in the order they are added.
    ///
    /// Args:
    ///     middleware (callable): A function that will process requests before route handlers.
    ///
    /// Returns:
    ///     Router: The router instance, allowing for method chaining.
    ///
    /// Example:
    /// ```python
    /// router = Router()
    ///     .route(get("/health", lambda _: "OK"))
    ///     .middleware(session_middleware)
    ///     .middleware(db_middleware)
    ///     .route(get("/login", login))
    ///     .middleware(auth_middleware)
    ///     .route(get("/dashboard", dashboard))
    /// ```
    fn middleware(mut slf: PyRefMut<'_, Self>, middleware: Py<PyAny>) -> PyRefMut<'_, Self> {
        let middleware = Middleware::new(middleware, slf.count);
        slf.count += 1;
        slf.middlewares.push(middleware);
        slf
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
    fn route(&mut self, mut route: Route) -> PyResult<Self> {
        route.sequence = self.count;
        let method_router = self.routes.entry(route.method.clone()).or_default();
        self.count += 1;

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
    fn routes(mut slf: PyRefMut<'_, Self>, routes: Vec<Route>) -> PyResult<PyRefMut<'_, Self>> {
        for route in routes {
            slf.route(route)?;
        }
        Ok(slf)
    }

    fn __repr__(&self) -> String {
        format!("{:#?}", self)
    }
}
