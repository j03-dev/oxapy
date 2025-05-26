use pyo3::{
    types::{PyAnyMethods, PyDict},
    Py, PyResult, Python,
};
use tokio::sync::mpsc::Receiver;

use crate::{
    into_response::convert_to_response, middleware::MiddlewareChain, request::Request,
    response::Response, routing::Router, serializer::ValidationException, status::Status,
    MatchRouteInfo, ProcessRequest, Wrap,
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
                    process_request.route_info,
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
    route_info: MatchRouteInfo,
    request: &Request,
) -> PyResult<Response> {
    Python::with_gil(|py| {
        let params = route_info.params;
        let route = route_info.route;

        let params_dict: Py<PyDict> = Wrap(params).into();
        let kwargs = params_dict.into_bound(py);

        kwargs.set_item("request", request.clone())?;

        let result = if !router.middlewares.is_empty() {
            let chain = MiddlewareChain::new(router.middlewares.clone());
            chain.execute(py, &route.handler.clone(), kwargs.clone())?
        } else {
            route.handler.call(py, (), Some(&kwargs))?
        };

        convert_to_response(result, py)
    })
}
