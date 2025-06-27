use std::sync::Arc;

use pyo3::{
    exceptions::PyValueError,
    types::{PyAnyMethods, PyDict, PyInt, PyString},
    Bound, PyObject, PyResult, Python,
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
    py: Python<'_>,
) {
    loop {
        tokio::select! {
            // handle `process_request` send by request handler
            Some(process_request) = request_receiver.recv() => {
                let mut response = process_response(
                        process_request.router,
                        process_request.match_route,
                        &process_request.request,
                        py,
                    ).unwrap_or_else(|err| {
                        let status = if err.is_instance_of::<ValidationException>(py)
                            { Status::BAD_REQUEST } else { Status::INTERNAL_SERVER_ERROR };
                        let response: Response = status.into();
                        response.set_body(err.to_string())
                });

                if let Some(catchers) = process_request.catchers {
                    if let Some(handler) = catchers.get(&response.status)  {
                        // TODO: handler the errors
                        let request: Request = process_request.request.as_ref().clone();
                        let result = handler.call(py, (request, response), None).unwrap();
                        response = convert_to_response(result, py).unwrap();
                    }
                }

                if let (Some(session), Some(store)) =
                (&process_request.request.session, &process_request.request.session_store)
                {
                    let cookie_header = store.get_cookie_header(session);
                    response.insert_or_append_cookie(cookie_header);
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

fn prepare_route_params<'py>(
    params: matchit::Params,
    py: Python<'py>,
) -> PyResult<Bound<'py, PyDict>> {
    let kwargs = PyDict::new(py);

    for (key, value) in params.iter() {
        match key.split_once(":") {
            Some((name, ty)) => {
                let parsed_value: PyObject = match ty {
                    "int" => PyInt::new(py, value.parse::<i64>()?).into(),
                    "str" => PyString::new(py, value).into(),
                    other => {
                        return Err(PyValueError::new_err(format!(
                            "Unsupported type annotation '{other}' in parameter key '{key}'."
                        )));
                    }
                };
                kwargs.set_item(name, parsed_value)?;
            }
            None => kwargs.set_item(key, value)?,
        }
    }

    Ok(kwargs)
}

fn process_response(
    router: Option<Arc<Router>>,
    match_route: Option<MatchRoute>,
    request: &Request,
    py: Python<'_>,
) -> PyResult<Response> {
    if let (Some(route), Some(router)) = (match_route, router) {
        let params = route.params;
        let route = route.value;

        let kwargs = prepare_route_params(params, py)?;
        let request = request.clone();

        let result = if !router.middlewares.is_empty() {
            let chain = MiddlewareChain::new(router.middlewares.clone());
            chain.execute(py, &route.handler.clone(), (request,), kwargs.clone())?
        } else {
            route.handler.call(py, (request,), Some(&kwargs))?
        };
        convert_to_response(result, py)
    } else {
        Ok(Status::NOT_FOUND.into())
    }
}
