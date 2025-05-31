use pyo3::{
    exceptions::PyValueError,
    types::{PyAnyMethods, PyDict, PyInt, PyString},
    PyObject, PyResult, Python,
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
            // handle `process_request` send by request handler
            Some(process_request) = request_receiver.recv() => {
                let mut response = Python::with_gil(|py| {
                    process_response(
                        &process_request.router,
                        process_request.match_route,
                        &process_request.request,
                        py,
                    ).unwrap_or_else(|err| {
                        let status = if err.is_instance_of::<ValidationException>(py)
                            { Status::BAD_REQUEST } else { Status::INTERNAL_SERVER_ERROR };
                        let response: Response = status.into();
                        response.set_body(err.to_string())
                    })
                });

                if let (Some(session), Some(store)) =
                (&process_request.request.session, &process_request.request.session_store)
                {
                    response.set_session_cookie(session, store);
                }

               if let Some(cors) = process_request.cors {
                    response = cors.apply_to_response(response).unwrap()
                }

                 // send back the response to the request handler
                _ = process_request.response_sender.send(response).await;
            }
            _ = shutdown_rx.recv() => {break}
        }
    }
}

fn process_response(
    router: &Router,
    match_route: MatchRoute,
    request: &Request,
    py: Python<'_>,
) -> PyResult<Response> {
    let params = match_route.params;
    let route = match_route.value;

    let kwargs = PyDict::new(py);

    for (key, value) in params.iter() {
        if let Some((name, ty)) = key.split_once(":") {
            let parsed_value: PyObject = match ty {
                "int" => {
                    let n = value.parse::<i64>().map_err(|_| {
                        PyValueError::new_err(format!(
                            "Failed to parse parameter '{key}' with value '{value}' as type 'int'."
                        ))
                    })?;
                    PyInt::new(py, n).into()
                }
                "str" => PyString::new(py, value).into(),
                other => {
                    return Err(PyValueError::new_err(format!(
                        "Unsupported type annotation '{other}' in parameter key '{key}'."
                    )));
                }
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
