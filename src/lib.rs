mod handling;
mod into_response;
mod request;
mod response;
mod routing;
mod status;

use handling::request_handler::handle_request;
use handling::response_handler::handle_response;
use pyo3::exceptions::PyException;
use request::Request;
use response::Response;
use routing::{delete, get, patch, post, put, static_files, Route, Router};
use status::Status;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;

use matchit::Match;
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

use pyo3::prelude::*;

type MatchitRoute = &'static Match<'static, 'static, &'static Route>;

struct ProcessRequest {
    request: Request,
    router: Router,
    route: MatchitRoute,
    response_sender: Sender<Response>,
    app_data: Option<Arc<Py<PyAny>>>,
}

#[derive(Clone)]
#[pyclass]
struct HttpServer {
    addr: SocketAddr,
    routers: Vec<Router>,
    app_data: Option<Arc<Py<PyAny>>>,
    max_connections: Arc<Semaphore>,
    channel_capacity: usize,
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
        })
    }

    fn app_data(&mut self, app_data: Py<PyAny>) {
        self.app_data = Some(Arc::new(app_data))
    }

    fn attach(&mut self, router: PyRef<'_, Router>) {
        self.routers.push(router.clone());
    }

    fn run(&self) -> PyResult<()> {
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(async move { self.run_server().await })?;
        Ok(())
    }

    #[pyo3(signature=(max_connections = 100, channel_capacity = 100))]
    fn config(&mut self, max_connections: usize, channel_capacity: usize) -> PyResult<()> {
        self.max_connections = Arc::new(Semaphore::new(max_connections));
        self.channel_capacity = channel_capacity;
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

        ctrlc::set_handler(move || {
            println!("\nReceived Ctrl+C! Shutting Down...");
            r.store(false, Ordering::SeqCst);
        })
        .ok();

        let listener = TcpListener::bind(addr).await?;
        println!("Listening on {}", addr);

        let routers = self.routers.clone();
        let running_clone = running.clone();
        let request_sender = request_sender.clone();
        let max_connections = self.max_connections.clone();
        let app_data = self.app_data.clone();

        tokio::spawn(async move {
            while running_clone.load(Ordering::SeqCst) {
                let permit = max_connections.clone().acquire_owned().await.unwrap();
                let (stream, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(stream);
                let request_sender = request_sender.clone();
                let routers = routers.clone();
                let app_data = app_data.clone();

                tokio::spawn(async move {
                    let _permit = permit;
                    http1::Builder::new()
                        .serve_connection(
                            io,
                            service_fn(move |req| {
                                let request_sender = request_sender.clone();
                                let routers = routers.clone();
                                let app_data = app_data.clone();
                                async move {
                                    handle_request(
                                        req,
                                        request_sender,
                                        routers,
                                        app_data,
                                        channel_capacity,
                                    )
                                    .await
                                }
                            }),
                        )
                        .await
                        .map_err(|err| {
                            PyException::new_err(format!("Error serving connection {err}"))
                        })?;

                    Ok::<(), PyErr>(())
                });
            }
        });

        handle_response(running, &mut request_receiver).await;

        Ok(())
    }
}

#[pymodule]
fn oxhttp(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<HttpServer>()?;
    m.add_class::<Router>()?;
    m.add_class::<Status>()?;
    m.add_class::<Response>()?;
    m.add_class::<Request>()?;
    m.add_function(wrap_pyfunction!(get, m)?)?;
    m.add_function(wrap_pyfunction!(post, m)?)?;
    m.add_function(wrap_pyfunction!(delete, m)?)?;
    m.add_function(wrap_pyfunction!(patch, m)?)?;
    m.add_function(wrap_pyfunction!(put, m)?)?;
    m.add_function(wrap_pyfunction!(static_files, m)?)?;

    Ok(())
}
