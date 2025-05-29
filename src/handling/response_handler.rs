use pyo3::{
    types::{PyAnyMethods, PyDict, PyInt, PyString},
    PyObject, PyResult, Python,
};
use tokio::sync::mpsc::Receiver;

use crate::{
    into_response::convert_to_response, middleware::MiddlewareChain, request::Request,
    response::Response, routing::Router, serializer::ValidationException, status::Status,
    IntoPyException, MatchRoute, ProcessRequest,
};

pub async fn handle_response(
    shutdown_rx: &mut Receiver<()>,
    request_receiver: &mut Receiver<ProcessRequest>,
) {
    loop {
        tokio::select! {
            Some(process_request) = request_receiver.recv() => {
                let mut response = Python::with_gil(|py| {
                    process_response(
                        &process_request.router,
                        process_request.route_info,
                        &process_request.request,
                        py,
                    ).unwrap_or_else(|err| {
                        let status = if err.is_instance_of::<ValidationException>(py)
                            { Status::BAD_REQUEST } else { Status::INTERNAL_SERVER_ERROR };
                        let response: Response = status.into();
                        response.set_body(err.to_string())
                    })
                });

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
    route_info: MatchRoute,
    request: &Request,
    py: Python<'_>,
) -> PyResult<Response> {
    let params = route_info.params;
    let route = route_info.value;

    let kwargs = PyDict::new(py);

    for (key, value) in params.iter() {
        if let Some((name, ty)) = key.split_once(":") {
            let parsed_value: PyObject = match ty {
                "int" => {
                    let n = value.parse::<i64>().into_py_exception()?;
                    PyInt::new(py, n).into()
                }
                "str" => PyString::new(py, value).into(),
                other => panic!("{other} is not supported"),
            };
            kwargs.set_item(name, parsed_value)?;
        } else {
            kwargs.set_item(key, value)?;
        }
    }

    kwargs.set_item("request", request.clone())?;

    let result = if !router.middlewares.is_empty() {
        let chain = MiddlewareChain::new(router.middlewares.clone());
        chain.execute(py, &route.handler.clone(), kwargs.clone())?
    } else {
        route.handler.call(py, (), Some(&kwargs))?
    };

    convert_to_response(result, py)
}
