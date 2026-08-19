#![allow(unused_variables, non_snake_case)]

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use cors::Cors;
use exceptions::IntoPyException;
use into_response::convert_to_response;
use middleware::MiddlewareChain;
use multipart::File;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::{PyBytes, PyDict, PyInt, PyString};
use pyo3_async_runtimes::tokio::{future_into_py, into_future};
use pyo3_stub_gen::derive::*;
use regex::Regex;
use request::{Request, RequestBuilder};
use response::{FileStreaming, Redirect, Response};
use routing::*;
use status::Status;
use templating::Template;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{
    Semaphore,
    mpsc::{Receiver, Sender, channel},
    oneshot,
};
use unicode_normalization::UnicodeNormalization;

use crate::middleware::Middleware;

mod cors;
#[macro_use]
mod exceptions;
mod into_response;
mod json;
mod jwt;
mod middleware;
mod multipart;
mod request;
mod response;
mod routing;
mod serializer;
mod status;
mod templating;

pyo3_stub_gen::export_verbatim!("oxapy", "from typing_extensions import Self");
pyo3_stub_gen::define_stub_info_gatherer!(stub_info);

struct ProcessRequest {
    match_route: Option<OwnedMatchRoute>,
    middlewares: Option<Arc<[Middleware]>>,
    request: Arc<Request>,
    response_sender: oneshot::Sender<Response>,
    cors: Option<Arc<Cors>>,
    wrapper: Option<Arc<Py<PyAny>>>,
}

#[derive(Clone)]
struct Context {
    app_data: Option<Arc<Py<PyAny>>>,
    request_sender: Sender<ProcessRequest>,
    routers: Vec<Arc<Router>>,
    template: Option<Arc<Template>>,
    wrapper: Option<Arc<Py<PyAny>>>,
    cors: Option<Arc<Cors>>,
}

struct ShutDownSignal {
    rx: Receiver<()>,
}

impl ShutDownSignal {
    fn new() -> PyResult<Self> {
        let running = Arc::new(AtomicBool::new(true));
        let (tx, rx) = channel::<()>(1);
        ctrlc::set_handler(move || {
            println!("\nShutting Down...");
            running.store(false, Ordering::SeqCst);
            let _ = block_on(tx.send(()), None);
        })
        .into_py_exception()?;
        Ok(Self { rx })
    }

    async fn wait(&mut self) {
        self.rx.recv().await;
    }
}

/// HTTP Server for handling web requests.
///
/// The HttpServer is the main entry point for creating web applications with OxAPY.
/// It manages routers, middleware, templates, sessions, and other components.
///
/// Args:
///     addr (tuple): A tuple containing the IP address and port to bind to.
///
/// Returns:
///     HttpServer: A new server instance.
///
/// Example:
/// ```python
/// from oxapy import HttpServer, Router, get, post
///
/// # Create a server on localhost port 8000
/// app = HttpServer(("127.0.0.1", 8000))
///
/// # Create a router
/// router = Router()
///
/// # Define route handlers using decorators
/// @get("/")
/// def home(request):
///     return "Hello, World!"
///
/// @get("/users/{user_id}")
/// def get_user(request, user_id: int):
///     return {"user_id": user_id, "name": f"User {user_id}"}
///
/// @post("/api/data")
/// def create_data(request):
///     # Access JSON data from the request
///     data = request.json()
///     return {"status": "success", "received": data}
///
/// # Register the routes with the router
/// router.routes([home, get_user, create_data])
///
/// # Attach the router to the server
/// app.attach(router)
///
/// # Run the server
/// app.run()
///     ```
#[gen_stub_pyclass]
#[pyclass(from_py_object, subclass)]
#[derive(Clone)]
struct HttpServer {
    addr: SocketAddr,
    app_data: Option<Arc<Py<PyAny>>>,
    wrapper: Option<Arc<Py<PyAny>>>,
    channel_capacity: usize,
    cors: Option<Arc<Cors>>,
    is_async: bool,
    routers: Vec<Arc<Router>>,
    max_connections: Arc<Semaphore>,
    template: Option<Arc<Template>>,
    running: Arc<AtomicBool>,
}

#[gen_stub_pyclass]
#[pyclass(subclass, extends=HttpServer)]
struct Oxapy;

#[gen_stub_pymethods]
#[pymethods]
impl Oxapy {
    #[new]
    #[gen_stub(override_return_type(type_repr = "typing_extensions.Self", imports = ("typing_extensions",)))]
    fn new(addr: (String, u16)) -> PyClassInitializer<Self> {
        todo!("dummy init")
    }

    #[pyo3(signature=(reload = false, workers = None))]
    fn run(&self, reload: bool, workers: Option<usize>) -> Py<PyAny> {
        todo!("dummy fonction")
    }

    fn set_patterns(&self, p: Vec<String>) -> PyRef<'_, Self> {
        todo!("dummy set_pattern method")
    }

    fn set_watch_dir(&self, dir: &str) -> PyRef<'_, Self> {
        todo!("dummy set_watch_dir fonction")
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl HttpServer {
    /// Create a new instance of HttpServer.
    ///
    /// Args:
    ///     addr (tuple): A tuple containing (ip_address: str, port: int)
    ///
    /// Returns:
    ///     HttpServer: A new server instance ready to be configured.
    ///
    /// Example:
    /// ```python
    /// server = HttpServer(("127.0.0.1", 5555))
    /// ```
    #[new]
    #[gen_stub(override_return_type(type_repr = "typing_extensions.Self", imports = ("typing_extensions",)))]
    fn new(addr: (String, u16)) -> PyResult<Self> {
        let (ip, port) = addr;
        Ok(Self {
            addr: SocketAddr::new(ip.parse()?, port),
            app_data: None,
            wrapper: None,
            channel_capacity: 100,
            cors: None,
            is_async: false,
            routers: Vec::new(),
            max_connections: Arc::new(Semaphore::new(100)),
            template: None,
            running: Arc::new(AtomicBool::new(true)),
        })
    }

    /// Set application-wide data that will be available to all request handlers.
    ///
    /// This is the perfect place to store shared resources like database connection pools,
    /// counters, or any other data that needs to be accessible across your application.
    ///
    /// Args:
    ///     app_data (any): Any Python object to be stored as application data.
    ///
    /// Returns:
    ///     Self
    ///
    /// Example:
    /// ```python
    /// from oxapy import get
    ///
    /// class AppState:
    ///     def __init__(self):
    ///         self.counter = 0
    ///         # You can store database connection pools here
    ///         self.db_pool = create_database_pool()
    ///
    /// app = HttpServer(("127.0.0.1", 5555))
    /// app.app_data(AppState())
    ///
    /// # Example of a handler that increments the counter
    /// @get("/counter")
    /// def increment_counter(request):
    ///     state = request.app_data
    ///     state.counter += 1
    ///     return {"count": state.counter}
    /// ```
    fn app_data(mut slf: PyRefMut<'_, Self>, app_data: Py<PyAny>) -> PyRefMut<'_, Self> {
        slf.app_data = Some(Arc::new(app_data));
        slf
    }

    /// Attach a router to the server.
    ///
    /// Multiple routers can be attached and are checked in order until a matching route is found.
    /// This is the recommended way to group routes with different middleware.
    ///
    /// Args:
    ///     router (Router): The router instance to attach.
    ///
    /// Returns:
    ///     Self
    ///
    /// Example:
    /// ```python
    /// from oxapy import Router, get, post
    ///
    /// @get("/")
    /// def hello(request):
    ///     return "Hello, World!"
    ///
    /// @get("/users/{user_id}")
    /// def get_user(request, user_id: int):
    ///     return f"User ID: {user_id}"
    ///
    /// @post("/api/data")
    /// def get_data(request):
    ///     return {"message": "Success", "data": [1, 2, 3]}
    ///
    /// router = Router()
    /// router.routes([hello, get_user, get_data])
    /// server.attach(router)
    /// ```
    fn attach(mut slf: PyRefMut<'_, Self>, router: Router) -> PyRefMut<'_, Self> {
        slf.routers.push(Arc::new(router));
        slf
    }

    /// Enable template rendering for the server.
    ///
    /// Args:
    ///     template (Template): An instance of Template for rendering HTML.
    ///
    /// Returns:
    ///     Self
    ///
    /// Example:
    /// ```python
    /// from oxapy import templating
    ///
    /// server.template(templating.Template())
    /// ```
    fn template(mut slf: PyRefMut<'_, Self>, template: Template) -> PyRefMut<'_, Self> {
        slf.template = Some(Arc::new(template));
        slf
    }

    /// Set up Cross-Origin Resource Sharing (CORS) for the server.
    ///
    /// Args:
    ///     cors (Cors): An instance of Cors with your desired CORS configuration.
    ///
    /// Returns:
    ///     Self
    ///
    /// Example:
    /// ```python
    /// cors = Cors()
    /// cors.origins = ["https://example.com"]
    /// server.cors(cors)
    /// ```
    fn cors(mut slf: PyRefMut<'_, Self>, cors: Cors) -> PyRefMut<'_, Self> {
        slf.cors = Some(Arc::new(cors));
        slf
    }

    /// Set the maximum number of concurrent connections the server will handle.
    ///
    /// Args:
    ///     max_connections (int): Maximum number of concurrent connections.
    ///
    /// Returns:
    ///     Self
    ///
    /// Example:
    /// ```python
    /// server.max_connections(1000)
    /// ```
    fn max_connections(mut slf: PyRefMut<'_, Self>, max_connections: usize) -> PyRefMut<'_, Self> {
        slf.max_connections = Arc::new(Semaphore::new(max_connections));
        slf
    }

    /// Set the internal channel capacity for handling requests.
    ///
    /// This is an advanced setting that controls how many pending requests
    /// can be buffered internally.
    ///
    /// Args:
    ///     channel_capacity (int): The channel capacity.
    ///
    /// Returns:
    ///     Self
    ///
    /// Example:
    /// ```python
    /// server.channel_capacity(200)
    /// ```
    fn channel_capacity(
        mut slf: PyRefMut<'_, Self>,
        channel_capacity: usize,
    ) -> PyRefMut<'_, Self> {
        slf.channel_capacity = channel_capacity;
        slf
    }

    /// Add a global wrapper (middleware) to the server.
    ///
    /// The wrapper is invoked with `(request, response)` after every handler.
    /// Its return value is converted like a handler's return value.
    ///
    /// Pipeline order: handler -> wrapper -> CORS.
    ///
    /// Args:
    ///     wrapper (callable): A function taking (request, response) as arguments.
    ///
    /// Returns:
    ///     Server: The server instance for method chaining.
    ///
    /// Example:
    /// ```python
    /// def global_middleware(request, response):
    ///     if response.status.code == 404:
    ///         return Response("<h1>Page Not Found</h1>", content_type="text/html")
    ///     return response
    ///
    /// server.wrap(global_middleware)
    /// ```
    fn wrap<'py>(
        mut slf: PyRefMut<'py, Self>,
        wrapper: Py<PyAny>,
        py: Python<'py>,
    ) -> PyRefMut<'py, Self> {
        slf.wrapper = Some(Arc::new(wrapper));
        slf
    }

    /// Enable asynchronous mode for the server.
    ///
    /// In asynchronous mode, request handlers can be asynchronous Python functions
    /// (i.e., defined with `async def`). This allows you to perform non-blocking
    /// I/O operations within your handlers.
    ///
    /// Returns:
    ///     HttpServer: A new HttpServer instance configured for asynchronous operation.
    ///
    /// Example:
    /// ```python
    /// import asyncio
    /// from oxapy import get, Router, HttpServer
    ///
    /// app = HttpServer(("127.0.0.1", 8000))
    /// router = Router()
    ///
    /// @get("/")
    /// async def home(request):
    ///     # Asynchronous operations are allowed here
    ///     data = await fetch_data_from_database()
    ///     return "Hello, World!"
    ///
    /// router.route(home)
    /// app.attach(router)
    ///
    /// async def main():
    ///     await app.async_mode().run()
    ///
    /// asyncio.run(main())
    /// ```
    fn async_mode(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.is_async = true;
        slf
    }

    /// Run the HTTP server.
    ///
    /// This starts the server and blocks until interrupted (e.g., with Ctrl+C).
    ///
    /// Args:
    ///     workers (int, optional): Number of worker threads to use. If not specified,
    ///                              the Tokio runtime will decide automatically.
    ///
    /// Returns:
    ///     None
    ///
    /// Example:
    /// ```python
    /// # Run with default number of workers
    /// server.run()
    ///
    /// # Or specify number of workers based on CPU count
    /// import multiprocessing
    /// workers = multiprocessing.cpu_count()
    /// server.run(workers)
    /// ```
    #[pyo3(signature=(workers=None))]
    fn run<'py>(&self, workers: Option<usize>, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let server = self.clone();

        if self.is_async {
            future_into_py(py, async move { server.run_server().await })
        } else {
            py.detach(move || block_on(server.run_server(), workers))?;
            Ok(py.None().into_bound(py))
        }
    }
}

impl HttpServer {
    async fn run_server(&self) -> PyResult<()> {
        let listener = TcpListener::bind(self.addr).await?;
        println!("Listening on {}", self.addr);
        let shutdown = ShutDownSignal::new()?;

        let (request_sender, request_receiver) = channel::<ProcessRequest>(self.channel_capacity);
        let ctx = Context {
            app_data: self.app_data.clone(),
            request_sender,
            routers: self.routers.clone(),
            template: self.template.clone(),
            wrapper: self.wrapper.clone(),
            cors: self.cors.clone(),
        };

        self.spawn_connection_handler(listener, Arc::new(ctx)).await;
        self.process_requests(shutdown, request_receiver).await
    }

    async fn spawn_connection_handler(&self, listener: TcpListener, ctx: Arc<Context>) {
        let running = self.running.clone();
        let max_connection = self.max_connections.clone();
        tokio::spawn(async move {
            while running.load(Ordering::SeqCst) {
                let permit = max_connection.clone().acquire_owned().await.unwrap();
                if let Ok((stream, _)) = listener.accept().await {
                    let _ = stream.set_nodelay(true);
                    let io = hyper_util::rt::TokioIo::new(stream);
                    Self::spawn_request_handler(io, ctx.clone(), permit);
                }
            }
        });
    }

    fn spawn_request_handler(
        io: hyper_util::rt::TokioIo<TcpStream>,
        ctx: Arc<Context>,
        _permit: tokio::sync::OwnedSemaphorePermit,
    ) {
        tokio::spawn(async move {
            hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new())
                .serve_connection(
                    io,
                    hyper::service::service_fn(move |req| {
                        let ctx = ctx.clone();
                        async move {
                            RequestBuilder::new(req)
                                .with_app_data(&ctx.app_data)
                                .with_template(&ctx.template)
                                .build()
                                .await
                                .unwrap()
                                .process(ctx)
                                .await
                        }
                    }),
                )
                .await
                .ok();
        });
    }

    async fn process_requests(
        &self,
        mut shutdown: ShutDownSignal,
        mut request_receiver: Receiver<ProcessRequest>,
    ) -> PyResult<()> {
        loop {
            tokio::select! {
                Some(pr) = request_receiver.recv() => {
                    let response = call_python_handler(&pr.middlewares, &pr.match_route, &pr.request, self.is_async)
                        .await
                        .unwrap_or_else(Response::from)
                        .call_wrapper(&pr)
                        .apply_cors(&pr.cors)?;
                    let _ = pr.response_sender.send(response);
                },
                _ = shutdown.wait() => break,
            }
        }
        Ok(())
    }
}

async fn call_python_handler(
    middlewares: &Option<Arc<[Middleware]>>,
    match_route: &Option<OwnedMatchRoute>,
    request: &Request,
    is_async: bool,
) -> PyResult<Response> {
    if let Some(match_route) = match_route {
        let mut result = Python::attach(|py| {
            let route = &match_route.value;
            let params = &match_route.params;
            let kwargs = build_route_params(py, params)?;

            match middlewares {
                Some(middlewares) => MiddlewareChain::execute(
                    py,
                    middlewares,
                    route.sequence,
                    &route.handler,
                    (request.clone(),),
                    kwargs,
                ),
                None => route.handler.call(py, (request.clone(),), Some(&kwargs)),
            }
        })?;

        if is_async {
            result = Python::attach(|py| into_future(result.into_bound(py)))?.await?;
        }

        Python::attach(|py| into_response::convert_to_response(result, py))
    } else {
        Ok(Status::NOT_FOUND.into())
    }
}

fn build_route_params<'py>(
    py: Python<'py>,
    params: &[(String, String)],
) -> PyResult<Bound<'py, PyDict>> {
    let kwargs = PyDict::new(py);
    for (key, value) in params.iter() {
        match key.split_once(':') {
            Some((name, ty)) => {
                let parsed = parse_params_value(py, value, ty)?;
                kwargs.set_item(name, parsed)?;
            }
            _ => kwargs.set_item(key, value)?,
        }
    }
    Ok(kwargs)
}

fn parse_params_value<'py>(py: Python<'py>, value: &str, ty: &str) -> PyResult<Bound<'py, PyAny>> {
    match ty {
        "int" => Ok(PyInt::new(py, value.parse::<i64>()?).into_any()),
        "str" => Ok(PyString::new(py, value).into_any()),
        "slug" => {
            static RE: PyOnceLock<Regex> = PyOnceLock::new();
            let re = RE
                .get_or_try_init(py, || Regex::new(r"[^a-z0-9]+"))
                .into_py_exception()?;
            let normalized: String = value.nfkd().filter(|c| c.is_ascii()).collect();
            let lowered = normalized.trim().to_lowercase();
            let replace = re.replace_all(&lowered, "-");
            Ok(PyString::new(py, replace.trim_matches('-')).into_any())
        }
        other => Err(PyValueError::new_err(format!(
            "Unsupported type annotation {other} in parameter"
        ))),
    }
}

fn block_on<F: std::future::Future>(future: F, workers: Option<usize>) -> F::Output {
    let mut runtime = tokio::runtime::Builder::new_multi_thread();
    workers.map(|w| runtime.worker_threads(w));
    runtime.enable_all().build().unwrap().block_on(future)
}

#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature=(path="/static", directory="./static"))]
fn static_file(path: &str, directory: &str) -> Route {
    // the implementation of this function is in __init__.py
    todo!("dummy static_file function")
}

#[gen_stub_pyfunction]
#[pyfunction]
fn send_file(path: &str) -> Response {
    // the implementation of this function is in __init__.py
    todo!("dummy send_file function")
}

#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature=(secret, max_age = 3600 * 24 * 7))]
fn Session(secret: Py<PyBytes>, max_age: i32) -> Py<PyAny> {
    // the implementation of this function is in __init__.py
    todo!("dummy session_middleware fonction")
}

#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature=(secret, cookie_name = "csrf_token", header_name = "x-csrf-token", field_name = "_csrf_token", cookie_max_age = 3600))]
fn CsrfProtect(
    secret: Py<PyBytes>,
    cookie_name: &str,
    header_name: &str,
    field_name: &str,
    cookie_max_age: i32,
) -> Py<PyAny> {
    // the implementation of this function is in __init__.py
    todo!("dummy CsrfProtect function")
}

#[pymodule]
fn oxapy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Cors>()?;
    m.add_class::<File>()?;
    m.add_class::<FileStreaming>()?;
    m.add_class::<HttpServer>()?;
    m.add_class::<Oxapy>()?;
    m.add_class::<Redirect>()?;
    m.add_class::<Request>()?;
    m.add_class::<Response>()?;
    m.add_class::<Route>()?;
    m.add_class::<Router>()?;
    m.add_class::<Status>()?;
    m.add_function(wrap_pyfunction!(convert_to_response, m)?)?;
    m.add_function(wrap_pyfunction!(delete, m)?)?;
    m.add_function(wrap_pyfunction!(get, m)?)?;
    m.add_function(wrap_pyfunction!(head, m)?)?;
    m.add_function(wrap_pyfunction!(options, m)?)?;
    m.add_function(wrap_pyfunction!(patch, m)?)?;
    m.add_function(wrap_pyfunction!(post, m)?)?;
    m.add_function(wrap_pyfunction!(put, m)?)?;
    m.add_function(wrap_pyfunction!(send_file, m)?)?;
    m.add_function(wrap_pyfunction!(static_file, m)?)?;
    m.add_function(wrap_pyfunction!(Session, m)?)?;
    m.add_function(wrap_pyfunction!(CsrfProtect, m)?)?;

    exceptions::exceptions(m)?;
    jwt::jwt_submodule(m)?;
    serializer::serializer_submodule(m)?;
    templating::templating_submodule(m)?;

    Ok(())
}
