use pyo3::{
    types::{PyAnyMethods, PyDict},
    PyResult, Python,
};
use tokio::sync::mpsc::Receiver;

use crate::{
    into_response::convert_to_response, middleware::MiddlewareChain, request::Request,
    response::Response, routing::Router, serializer::ValidationException, status::Status,
    MatchRoute, ProcessRequest,
};

pub async fn handle_response(
    shutdown_rx: &mut Receiver<()>,
    request_receiver: &mut Receiver<ProcessRequest>,
) {
    loop {
        tokio::select! {
            Some(process_request) = request_receiver.recv() => {
                let mut response: Response = match process_response(
                    &process_request.router,
                    process_request.route,
                    &process_request.request,
                ) {
                    Ok(response) => response,
                    Err(err) => {
                        Python::with_gil(|py|{
                            let status = if err.is_instance_of::<ValidationException>(py)
                                { Status::BAD_REQUEST } else { Status::INTERNAL_SERVER_ERROR };
                            let response: Response = status.into();
                            response.set_body(err.to_string())
                        })
                    }
                };

                if let (Some(session), Some(store)) = (&process_request.request.session, &process_request.request.session_store) {
                    response.set_session_cookie(session, store);
                }

               if let Some(cors) = process_request.cors {
                    response = cors.apply_to_response(response).unwrap()
                }

                _ = process_request.response_sender.send(response).await;
            }
            _ = shutdown_rx.recv() => {break}
        }
    }
}

fn process_response(
    router: &Router,
    matchit_route: MatchRoute,
    request: &Request,
) -> PyResult<Response> {
    Python::with_gil(|py| {
        let kwargs = &PyDict::new(py);
        let params = &matchit_route.params;
        let route = matchit_route.value;

        for (key, value) in params.iter() {
            kwargs.set_item(key, value)?;
        }

        kwargs.set_item("request", request.clone())?;

        let result = if !router.middlewares.is_empty() {
            let chain = MiddlewareChain::new(router.middlewares.clone());
            chain.execute(py, &route.handler.clone(), kwargs.clone())?
        } else {
            route.handler.call(py, (), Some(kwargs))?
        };

        convert_to_response(result, py)
    })
}
