use hyper::{body::Bytes, header::CONTENT_TYPE, HeaderMap};
use pyo3::{prelude::*, types::PyAny, Py};

use crate::{
    cors::Cors, exceptions::IntoPyException, exceptions::*, json, response::ResponseBody,
    status::Status, Response,
};

type Error = Box<dyn std::error::Error>;

impl TryFrom<String> for Response {
    type Error = Error;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "text/plain".parse()?);
        Ok(Response {
            status: Status::OK,
            headers,
            body: ResponseBody::Bytes(val.into()),
        })
    }
}

impl TryFrom<Py<PyAny>> for Response {
    type Error = Error;

    fn try_from(val: Py<PyAny>) -> Result<Self, Self::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        Ok(Response {
            status: Status::OK,
            headers,
            body: ResponseBody::Bytes(json::dumps(&val)?.into()),
        })
    }
}

impl TryFrom<(String, Status)> for Response {
    type Error = Error;

    fn try_from(val: (String, Status)) -> Result<Self, Self::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "text/plain".parse()?);
        Ok(Response {
            status: val.1,
            headers,
            body: ResponseBody::Bytes(val.0.clone().into()),
        })
    }
}

impl TryFrom<(Py<PyAny>, Status)> for Response {
    type Error = Error;

    fn try_from(val: (Py<PyAny>, Status)) -> Result<Self, Self::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        Ok(Response {
            status: val.1,
            headers,
            body: ResponseBody::Bytes(json::dumps(&val.0)?.into()),
        })
    }
}

impl From<Status> for Response {
    fn from(val: Status) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        Response {
            status: val,
            headers,
            body: ResponseBody::Bytes(Bytes::new()),
        }
    }
}

impl From<PyErr> for Response {
    fn from(value: PyErr) -> Self {
        Python::attach(|py| {
            let status = match value.is_instance_of::<ClientError>(py) {
                true if value.is_instance_of::<UnauthorizedError>(py) => Status::UNAUTHORIZED,
                true if value.is_instance_of::<ForbiddenError>(py) => Status::FORBIDDEN,
                true if value.is_instance_of::<NotFoundError>(py) => Status::NOT_FOUND,
                true if value.is_instance_of::<ConflictError>(py) => Status::CONFLICT,
                true => Status::BAD_REQUEST,
                false => {
                    let debug = std::env::var("DEBUG")
                        .ok()
                        .and_then(|v| v.parse::<bool>().ok())
                        .unwrap_or(true);
                    if debug {
                        value.display(py);
                    }
                    Status::INTERNAL_SERVER_ERROR
                }
            };
            let response: Response = status.into();
            response.set_body(format!(
                r#"{{"detail": "{}"}}"#,
                value.value(py).to_string().replace('"', "'")
            ))
        })
    }
}

impl From<Cors> for Response {
    fn from(val: Cors) -> Self {
        let mut response = Status::NO_CONTENT.into();
        val.apply_headers(&mut response);
        response
    }
}

macro_rules! to_response {
    ($rslt:expr, $py:expr, $($type:ty),*) => {{
        $(
            if let Ok(value) = $rslt.extract::<$type>($py) {
                return value.try_into().into_py_exception();
            }
        )*

        return Err(pyo3::exceptions::PyException::new_err(
            "Failed to convert this type to response",
        ));
    }};
}

/// Convert a Python object into an OxAPY `Response`.
///
/// This function normalizes the output of Python route handlers into a `Response` object
/// that can be sent back to the client. It supports multiple return types such as strings,
/// dictionaries, tuples, or even existing `Response` objects.
///
/// If a handler returns an unsupported type, a `PyValueError` will be raised.
///
/// Args:
///     result (Any): The Python object returned by a route handler or middleware.
///
/// Returns:
///     Response: A valid OxAPY `Response` object.
///
/// Supported return types:
///     - `Response`: Returned directly without modification.
///     - `str`: Converted into a text/plain `Response`.
///     - `dict` / Python object: Serialized into JSON automatically.
///     - `(str, Status)`: Returns a text response with the given status code.
///     - `(Response, Status)`: Returns the provided `Response` but updates the status code.
///
/// Example:
/// ```python
/// from oxapy import Response, Status, convert_to_response
///
/// # From string
/// convert_to_response("Hello World")
///
/// # From dictionary (converted to JSON)
/// convert_to_response({"message": "success"})
///
/// # From tuple (content + status)
/// convert_to_response(("Not Found", Status.NOT_FOUND))
///
/// # From Response instance
/// response = Response("OK", status=Status.OK)
/// convert_to_response(response)
/// ```
///
/// Notes:
///     This function is mainly used internally by the framework to unify handler return types,
///     but it can also be used manually if you’re building custom middlewares or decorators.
#[pyo3_stub_gen::derive::gen_stub_pyfunction]
#[pyfunction]
#[inline]
pub fn convert_to_response(result: Py<PyAny>, py: Python<'_>) -> PyResult<Response> {
    to_response!(
        result,
        py,
        Response,
        Status,
        (String, Status),
        (Py<PyAny>, Status),
        String,
        Py<PyAny>
    )
}
