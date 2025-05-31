use std::sync::Arc;
use std::{collections::HashMap, mem::transmute};

use http_body_util::{BodyExt, Full};
use hyper::{
    body::{Bytes, Incoming},
    Request as HyperRequest, Response as HyperResponse,
};
use pyo3::{Py, PyAny, PyResult};
use tokio::sync::mpsc::channel;

use crate::{
    multipart::{parse_mutltipart, MultiPart},
    request::Request,
    response::Response,
    session::SessionStore,
    status::Status,
    templating::Template,
    IntoPyException, MatchRoute, ProcessRequest, RequestContext,
};

fn convert_oxapy_response_to_hyper_response(
    response: Response,
) -> Result<HyperResponse<Full<Bytes>>, hyper::http::Error> {
    let mut response_builder = HyperResponse::builder().status(response.status as u16);
    for (key, value) in response.headers {
        response_builder = response_builder.header(key, value);
    }
    response_builder.body(Full::new(response.body))
}

fn extract_session_id_from_cookie(
    cookie_header: Option<&String>,
    cookie_name: &str,
) -> Option<String> {
    cookie_header.and_then(|cookies| {
        cookies
            .split(';')
            .filter_map(|cookie| {
                let cookie = cookie.trim();
                let mut parts = cookie.splitn(2, '=');
                if let (Some(name), Some(value)) = (
                    parts.next().map(|s| s.trim()),
                    parts.next().map(|s| s.trim()),
                ) {
                    if name == cookie_name {
                        return Some(value.to_string());
                    }
                }
                None
            })
            .next()
    })
}

fn setup_session_request(
    session_store: Option<Arc<SessionStore>>,
    mut request: Request,
) -> Result<Request, Box<dyn std::error::Error + Send + Sync>> {
    let headers = &request.headers;
    if let Some(ref store) = session_store {
        let session_id = extract_session_id_from_cookie(headers.get("cookie"), &store.cookie_name);

        let session = store.get_session(session_id).map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get session: {}", e),
            ))
        })?;
        request.session = Some(Arc::new(session));
        request.session_store = Some(store.clone());
    }
    Ok(request)
}

async fn setup_mutltpart_request(mut request: Request, body_bytes: Bytes) -> PyResult<Request> {
    let headers = &request.headers;
    if let Some(content_type) = headers.get("content-type") {
        if content_type.starts_with("multipart/form-data") {
            let MultiPart { fields, files } = parse_mutltipart(content_type, body_bytes)
                .await
                .into_py_exception()?;
            request.form = Some(fields);
            request.files = Some(files);
        }
    }
    Ok(request)
}

async fn convert_hyper_request_to_oxapy_request(
    req: HyperRequest<Incoming>,
    app_data: Option<Arc<Py<PyAny>>>,
    template: Option<Arc<Template>>,
    session_store: Option<Arc<SessionStore>>,
) -> Result<Arc<Request>, Box<dyn std::error::Error + Sync + Send>> {
    let method = req.method().to_string();
    let uri = req.uri().to_string();

    let mut headers = HashMap::new();
    for (key, value) in req.headers() {
        headers.insert(
            key.to_string(),
            value.to_str().unwrap_or_default().to_string(),
        );
    }

    let mut request = Request::new(method, uri, headers.clone());

    request = setup_session_request(session_store, request)?;

    let body_bytes = req.collect().await?.to_bytes();
    let body = String::from_utf8_lossy(&body_bytes).to_string();

    request = setup_mutltpart_request(request, body_bytes).await?;

    if !body.is_empty() {
        request.body = Some(body);
    }

    request.app_data = app_data;
    request.template = template;

    Ok(Arc::new(request))
}

pub async fn handle_request(
    req: HyperRequest<Incoming>,
    request_ctx: Arc<RequestContext>,
) -> Result<HyperResponse<Full<Bytes>>, hyper::http::Error> {
    let RequestContext {
        request_sender,
        routers,
        app_data,
        channel_capacity,
        cors,
        template,
        session_store,
    } = request_ctx.as_ref().clone();

    if req.method() == hyper::Method::OPTIONS && cors.is_some() {
        let response = cors.unwrap().as_ref().clone();
        return convert_oxapy_response_to_hyper_response(response.into());
    }

    let request = convert_hyper_request_to_oxapy_request(req, app_data, template, session_store)
        .await
        .unwrap();

    for router in &routers {
        if let Some(match_route) = router.find(&request.method, &request.uri) {
            let (response_sender, mut respond_receive) = channel(channel_capacity);

            let match_route: MatchRoute = unsafe { transmute(match_route) };

            let process_request = ProcessRequest {
                request,
                router: router.clone(),
                match_route,
                response_sender,
                cors: cors.clone(),
            };

            // send the `process_request` to response handler
            if request_sender.send(process_request).await.is_ok() {
                // handle response `back` from `response handler`
                if let Some(response) = respond_receive.recv().await {
                    return convert_oxapy_response_to_hyper_response(response);
                }
            }
            break;
        }
    }

    let response = if let Some(cors_config) = cors {
        cors_config
            .apply_to_response(Status::NOT_FOUND.into())
            .unwrap()
    } else {
        Status::NOT_FOUND.into()
    };

    convert_oxapy_response_to_hyper_response(response)
}
