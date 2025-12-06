mod catcher;
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
mod session;
mod status;
mod templating;

use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::catcher::Catcher;
use crate::cors::Cors;
use crate::exceptions::IntoPyException;
use crate::middleware::MiddlewareChain;
use crate::multipart::File;
use crate::request::{Request, RequestBuilder};
use crate::response::{FileStreaming, Redirect, Response};
use crate::routing::*;
use crate::session::{Session, SessionStore};
use crate::status::Status;
use crate::templating::Template;

use pyo3::exceptions::PyValueError;
use pyo3::types::{PyDict, PyInt, PyString};
use pyo3_async_runtimes::tokio::{future_into_py, into_future};
use pyo3_stub_gen::derive::*;

use ahash::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Semaphore;

use pyo3::prelude::*;

struct ProcessRequest {
    request: Arc<Request>,
    layer: Option<Arc<Layer>>,
    match_route: Option<MatchRoute<'static>>,
    tx: Sender<Response>,
    cors: Option<Arc<Cors>>,
    catchers: Option<Arc<HashMap<Status, Py<PyAny>>>>,
}

#[derive(Clone)]
struct RequestContext {
    app_data: Option<Arc<Py<PyAny>>>,
    catchers: Option<Arc<HashMap<Status, Py<PyAny>>>>,
    channel_capacity: usize,
    cors: Option<Arc<Cors>>,
    request_sender: Sender<ProcessRequest>,
    layers: Vec<Arc<Layer>>,
    session_store: Option<Arc<SessionStore>>,
    template: Option<Arc<Template>>,
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
#[pyclass]
#[derive(Clone)]
struct HttpServer {
    addr: SocketAddr,
    app_data: Option<Arc<Py<PyAny>>>,
    catchers: Option<Arc<HashMap<Status, Py<PyAny>>>>,
    channel_capacity: usize,
    cors: Option<Arc<Cors>>,
    is_async: bool,
    layers: Vec<Arc<Layer>>,
    max_connections: Arc<Semaphore>,
    session_store: Option<Arc<SessionStore>>,
    template: Option<Arc<Template>>,
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
    fn new(addr: (String, u16)) -> PyResult<Self> {
        let (ip, port) = addr;
        Ok(Self {
            addr: SocketAddr::new(ip.parse()?, port),
            app_data: None,
            catchers: None,
            channel_capacity: 100,
            cors: None,
            is_async: false,
            layers: Vec::new(),
            max_connections: Arc::new(Semaphore::new(100)),
            session_store: None,
            template: None,
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
    ///     None
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
    /// Args:
    ///     router (Router): The router instance to attach.
    ///
    /// Returns:
    ///     None
    ///
    /// Example:
    /// ```python
    /// from oxapy import Router, get, post
    ///
    /// # Define a simple hello world handler
    /// @get("/")
    /// def hello(request):
    ///     return "Hello, World!"
    ///
    /// # Handler with path parameters
    /// @get("/users/{user_id}")
    /// def get_user(request, user_id: int):
    ///     return f"User ID: {user_id}"
    ///
    /// # Handler that returns JSON
    /// @post("/api/data")
    /// def get_data(request):
    ///     return {"message": "Success", "data": [1, 2, 3]}
    ///
    /// router = Router()
    /// router.routes([hello, get_user, get_data])
    /// # Attach the router to the server
    /// server.attach(router)
    /// ```
    fn attach(mut slf: PyRefMut<'_, Self>, router: Router) -> PyRefMut<'_, Self> {
        let arc_layers = router.layers.into_iter().map(Arc::new);
        slf.layers.extend(arc_layers);
        slf
    }

    /// Set up a session store for managing user sessions.
    ///
    /// When configured, session data will be available in request handlers.
    ///
    /// Args:
    ///     session_store (SessionStore): The session store instance to use.
    ///
    /// Returns:
    ///     None
    ///
    /// Example:
    /// ```python
    /// server.session_store(SessionStore())
    /// ```
    fn session_store(
        mut slf: PyRefMut<'_, Self>,
        session_store: SessionStore,
    ) -> PyRefMut<'_, Self> {
        slf.session_store = Some(Arc::new(session_store));
        slf
    }

    /// Enable template rendering for the server.
    ///
    /// Args:
    ///     template (Template): An instance of Template for rendering HTML.
    ///
    /// Returns:
    ///     None
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
    ///     None
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
    ///     None
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
    ///     None
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

    /// Add status code catchers to the server.
    ///
    /// Args:
    ///     catchers (list): A list of Catcher handlers for specific status codes.
    ///
    /// Returns:
    ///     None
    ///
    /// Example:
    /// ```python
    /// @catcher(Status.NOT_FOUND)
    /// def not_found(request, response):
    ///     return Response("<h1>Page Not Found</h1>", content_type="text/html")
    ///
    /// server.catchers([not_found])
    /// ```
    fn catchers<'py>(
        mut slf: PyRefMut<'py, Self>,
        catchers: Vec<PyRef<Catcher>>,
        py: Python<'py>,
    ) -> PyRefMut<'py, Self> {
        let mut map = HashMap::default();

        for catcher in catchers {
            map.insert(catcher.status, catcher.handler.clone_ref(py));
        }

        slf.catchers = Some(Arc::new(map));
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
        let (listener, shutdown) = self.setup_serve().await?;
        let (ctx, rx) = self.create_request_context();

        self.spawn_connection_handler(listener, Arc::new(ctx)).await;
        self.process_requests(shutdown, rx).await
    }

    async fn setup_serve(&self) -> PyResult<(TcpListener, ShutDownSignal)> {
        let listener = TcpListener::bind(self.addr).await?;
        println!("Listening on {}", self.addr);

        let shutdown = ShutDownSignal::new()?;
        Ok((listener, shutdown))
    }

    fn create_request_context(&self) -> (RequestContext, Receiver<ProcessRequest>) {
        let (tx, rx) = channel::<ProcessRequest>(self.channel_capacity);
        let ctx = RequestContext {
            app_data: self.app_data.clone(),
            catchers: self.catchers.clone(),
            channel_capacity: self.channel_capacity,
            cors: self.cors.clone(),
            layers: self.layers.clone(),
            request_sender: tx,
            session_store: self.session_store.clone(),
            template: self.template.clone(),
        };
        (ctx, rx)
    }

    async fn spawn_connection_handler(&self, listener: TcpListener, ctx: Arc<RequestContext>) {
        let running = Arc::new(AtomicBool::new(true));
        let max_connection = self.max_connections.clone();

        tokio::spawn(async move {
            while running.load(Ordering::SeqCst) {
                let permit = max_connection.clone().acquire_owned().await.unwrap();
                if let Ok((stream, _)) = listener.accept().await {
                    let io = hyper_util::rt::TokioIo::new(stream);
                    Self::spawn_request_handler(io, ctx.clone(), permit);
                }
            }
        });
    }

    fn spawn_request_handler(
        io: hyper_util::rt::TokioIo<TcpStream>,
        ctx: Arc<RequestContext>,
        _permit: tokio::sync::OwnedSemaphorePermit,
    ) {
        tokio::spawn(async move {
            let http = hyper::server::conn::http1::Builder::new();
            let conn = http.serve_connection(
                io,
                hyper::service::service_fn(move |req| {
                    let ctx = ctx.clone();
                    async move {
                        let request = RequestBuilder::new(req)
                            .with_app_data(&ctx.app_data)
                            .with_template(&ctx.template)
                            .with_session_store(&ctx.session_store)
                            .build()
                            .await
                            .unwrap();
                        request.process(ctx).await
                    }
                }),
            );

            if let Err(err) = conn.await {
                eprintln!("server connection error: {err}!");
            }
        });
    }

    async fn process_requests(
        &self,
        mut shutdown: ShutDownSignal,
        mut rx: Receiver<ProcessRequest>,
    ) -> PyResult<()> {
        loop {
            tokio::select! {
                Some(req) = rx.recv() => self.handle_request(req).await?,
                _ = shutdown.wait() => break,
            }
        }
        Ok(())
    }

    async fn handle_request(&self, req: ProcessRequest) -> PyResult<()> {
        let mut response =
            call_python_handler(&req.layer, &req.match_route, &req.request, self.is_async)
                .await
                .unwrap_or_else(Response::from);

        response = self.apply_catcher(response, &req);
        response = self.apply_session(response, &req.request);
        response = self.apply_cors(response, &req.cors)?;

        let _ = req.tx.send(response).await;
        Ok(())
    }

    fn apply_catcher(&self, mut response: Response, req: &ProcessRequest) -> Response {
        if let Some(catchers) = &req.catchers {
            if let Some(handler) = catchers.get(&response.status) {
                let request: Request = req.request.as_ref().clone();
                response = Python::attach(|py| {
                    let result = handler.call(py, (request, response), None)?;
                    into_response::convert_to_response(result, py)
                })
                .unwrap_or_else(Response::from);
            }
        }
        response
    }

    fn apply_session(&self, mut response: Response, request: &Arc<Request>) -> Response {
        if let (Some(session), Some(store)) = (&request.session, &request.session_store) {
            let cookie_header = store.get_cookie_header(session);
            response.insert_or_append_cookie(cookie_header);
        }
        response
    }

    fn apply_cors(&self, mut response: Response, cors: &Option<Arc<Cors>>) -> PyResult<Response> {
        if let Some(cors) = cors {
            response = cors.apply_to_response(response)?;
        }
        Ok(response)
    }
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

async fn call_python_handler<'l>(
    layer: &Option<Arc<Layer>>,
    match_route: &Option<MatchRoute<'l>>,
    request: &Request,
    is_async: bool,
) -> PyResult<Response> {
    match (match_route, layer) {
        (Some(route), Some(layer)) => {
            let mut result = execute_route_handler(&route, &layer, request)?;

            if is_async {
                result = Python::attach(|py| into_future(result.into_bound(py)))?.await?;
            }

            Python::attach(|py| into_response::convert_to_response(result, py))
        }
        _ => Ok(Status::NOT_FOUND.into()),
    }
}

fn execute_route_handler(
    route: &MatchRoute,
    layer: &Layer,
    request: &Request,
) -> PyResult<Py<PyAny>> {
    Python::attach(|py| {
        let kwargs = build_route_params(py, &route.params)?;

        if layer.middlewares.is_empty() {
            route
                .value
                .handler
                .call(py, (request.clone(),), Some(&kwargs))
        } else {
            let chain = MiddlewareChain::new(layer.middlewares.clone());
            chain.execute(
                py,
                route.value.handler.deref(),
                (request.clone(),),
                kwargs.clone(),
            )
        }
    })
}

fn build_route_params<'py>(
    py: Python<'py>,
    params: &matchit::Params,
) -> PyResult<Bound<'py, PyDict>> {
    let kwargs = PyDict::new(py);

    for (key, value) in params.iter() {
        match key.split_once(":") {
            Some((name, ty)) => {
                let parsed = parse_params_value(py, value, ty)?;
                kwargs.set_item(name, parsed)?;
            }
            _ => kwargs.set_item(key, value)?,
        }
    }
    Ok(kwargs)
}

fn parse_params_value(py: Python<'_>, value: &str, ty: &str) -> PyResult<Py<PyAny>> {
    match ty {
        "int" => Ok(PyInt::new(py, value.parse::<i64>()?).into()),
        "str" => Ok(PyString::new(py, value).into()),
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

pyo3_stub_gen::define_stub_info_gatherer!(stub_info);

#[pymodule]
fn oxapy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<HttpServer>()?;
    m.add_class::<Router>()?;
    m.add_class::<Status>()?;
    m.add_class::<Response>()?;
    m.add_class::<Request>()?;
    m.add_class::<Cors>()?;
    m.add_class::<Session>()?;
    m.add_class::<SessionStore>()?;
    m.add_class::<Redirect>()?;
    m.add_class::<File>()?;
    m.add_class::<FileStreaming>()?;
    m.add_function(wrap_pyfunction!(get, m)?)?;
    m.add_function(wrap_pyfunction!(post, m)?)?;
    m.add_function(wrap_pyfunction!(delete, m)?)?;
    m.add_function(wrap_pyfunction!(patch, m)?)?;
    m.add_function(wrap_pyfunction!(put, m)?)?;
    m.add_function(wrap_pyfunction!(head, m)?)?;
    m.add_function(wrap_pyfunction!(options, m)?)?;
    m.add_function(wrap_pyfunction!(static_file, m)?)?;
    m.add_function(wrap_pyfunction!(catcher::catcher, m)?)?;
    m.add_function(wrap_pyfunction!(into_response::convert_to_response, m)?)?;

    json::init_orjson(m.py())?;
    templating::templating_submodule(m)?;
    serializer::serializer_submodule(m)?;
    exceptions::exceptions(m)?;
    jwt::jwt_submodule(m)?;

    Ok(())
}
