mod cors;
mod handling;
mod into_response;
mod json;
#[cfg(not(target_arch = "aarch64"))]
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

use cors::Cors;
use handling::request_handler::handle_request;
use handling::response_handler::handle_response;
use into_response::convert_to_response;
use multipart::File;
use pyo3::types::PyDict;
use request::Request;
use response::{Redirect, Response};
use routing::{delete, get, head, options, patch, post, put, static_file, Route, Router};
use serde::{Deserialize, Serialize};
use session::{Session, SessionStore};
use status::Status;
use templating::Template;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;

use tokio::net::TcpListener;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::Semaphore;

use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use pyo3::{exceptions::PyException, prelude::*};

type MatchRoute<'l> = matchit::Match<'l, 'l, &'l Route>;

trait IntoPyException<T> {
    fn into_py_exception(self) -> PyResult<T>;
}

impl<T, E: ToString> IntoPyException<T> for Result<T, E> {
    fn into_py_exception(self) -> PyResult<T> {
        self.map_err(|err| PyException::new_err(err.to_string()))
    }
}

struct Wrap<T>(T);

impl<T> From<Bound<'_, PyDict>> for Wrap<T>
where
    T: for<'de> Deserialize<'de>,
{
    fn from(value: Bound<'_, PyDict>) -> Self {
        let json_string = json::dumps(&value.into()).unwrap();
        let value = serde_json::from_str(&json_string).unwrap();
        Wrap(value)
    }
}

impl<T> From<Wrap<T>> for Py<PyDict>
where
    T: Serialize,
{
    fn from(value: Wrap<T>) -> Self {
        let json_string = serde_json::json!(value.0).to_string();
        json::loads(&json_string).unwrap()
    }
}

struct ProcessRequest {
    request: Arc<Request>,
    router: Arc<Router>,
    match_route: MatchRoute<'static>,
    response_sender: Sender<Response>,
    cors: Option<Arc<Cors>>,
}

#[derive(Clone)]
struct RequestContext {
    request_sender: Sender<ProcessRequest>,
    routers: Vec<Arc<Router>>,
    app_data: Option<Arc<Py<PyAny>>>,
    channel_capacity: usize,
    cors: Option<Arc<Cors>>,
    template: Option<Arc<Template>>,
    session_store: Option<Arc<SessionStore>>,
}

#[derive(Clone)]
#[pyclass]
struct HttpServer {
    addr: SocketAddr,
    routers: Vec<Arc<Router>>,
    app_data: Option<Arc<Py<PyAny>>>,
    max_connections: Arc<Semaphore>,
    channel_capacity: usize,
    cors: Option<Arc<Cors>>,
    template: Option<Arc<Template>>,
    session_store: Option<Arc<SessionStore>>,
}

#[pymethods]
impl HttpServer {
    #[new]
    fn new(addr: (String, u16)) -> PyResult<Self> {
        let (ip, port) = addr;
        Ok(Self {
            addr: SocketAddr::new(ip.parse()?, port),
            routers: Vec::new(),
            app_data: None,
            max_connections: Arc::new(Semaphore::new(100)),
            channel_capacity: 100,
            cors: None,
            template: None,
            session_store: None,
        })
    }

    fn app_data(&mut self, app_data: Py<PyAny>) {
        self.app_data = Some(Arc::new(app_data))
    }

    fn attach(&mut self, router: Router) {
        self.routers.push(Arc::new(router));
    }

    fn session_store(&mut self, session_store: SessionStore) {
        self.session_store = Some(Arc::new(session_store));
    }

    fn template(&mut self, template: Template) {
        self.template = Some(Arc::new(template))
    }

    fn cors(&mut self, cors: Cors) {
        self.cors = Some(Arc::new(cors));
    }

    fn max_connections(&mut self, max_connections: usize) {
        self.max_connections = Arc::new(Semaphore::new(max_connections));
    }

    fn channel_capacity(&mut self, channel_capacity: usize) {
        self.channel_capacity = channel_capacity;
    }

    #[pyo3(signature=(workers=None))]
    fn run(&self, workers: Option<usize>) -> PyResult<()> {
        let mut runtime = tokio::runtime::Builder::new_multi_thread();

        if let Some(workers) = workers {
            runtime.worker_threads(workers);
        }

        runtime
            .enable_all()
            .build()?
            .block_on(async move { self.run_server().await })?;

        Ok(())
    }
}

impl HttpServer {
    async fn run_server(&self) -> PyResult<()> {
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        let addr = self.addr;
        let channel_capacity = self.channel_capacity;

        let (request_sender, mut request_receiver) = channel::<ProcessRequest>(channel_capacity);
        let (shutdown_tx, mut shutdown_rx) = channel::<()>(1);

        ctrlc::set_handler(move || {
            println!("\nReceived Ctrl+C! Shutting Down...");
            r.store(false, Ordering::SeqCst);
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(shutdown_tx.send(())).unwrap();
        })
        .into_py_exception()?;

        let listener = TcpListener::bind(addr).await?;
        println!("Listening on {}", addr);

        let running_clone = running.clone();
        let max_connections = self.max_connections.clone();

        let request_ctx = Arc::new(RequestContext {
            routers: self.routers.clone(),
            request_sender: request_sender.clone(),
            app_data: self.app_data.clone(),
            cors: self.cors.clone(),
            template: self.template.clone(),
            session_store: self.session_store.clone(),
            channel_capacity,
        });

        tokio::spawn(async move {
            while running_clone.load(Ordering::SeqCst) {
                let permit = max_connections.clone().acquire_owned().await.unwrap();
                let (stream, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(stream);
                let request_ctx = request_ctx.clone();

                tokio::spawn(async move {
                    let _permit = permit;
                    http1::Builder::new()
                        .serve_connection(
                            io,
                            service_fn(move |req| {
                                let request_ctx = request_ctx.clone();
                                async move {
                                    handle_request(req, request_ctx).await // ping
                                }
                            }),
                        )
                        .await
                        .into_py_exception()
                });
            }
        });

        handle_response(&mut shutdown_rx, &mut request_receiver).await; // pong

        Ok(())
    }
}

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
    m.add_function(wrap_pyfunction!(get, m)?)?;
    m.add_function(wrap_pyfunction!(post, m)?)?;
    m.add_function(wrap_pyfunction!(delete, m)?)?;
    m.add_function(wrap_pyfunction!(patch, m)?)?;
    m.add_function(wrap_pyfunction!(put, m)?)?;
    m.add_function(wrap_pyfunction!(head, m)?)?;
    m.add_function(wrap_pyfunction!(options, m)?)?;
    m.add_function(wrap_pyfunction!(static_file, m)?)?;
    m.add_function(wrap_pyfunction!(convert_to_response, m)?)?;

    templating::templating_submodule(m)?;
    serializer::serializer_submodule(m)?;

    #[cfg(not(target_arch = "aarch64"))]
    jwt::jwt_submodule(m)?;

    Ok(())
}
