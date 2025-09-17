use hyper::{body::Bytes, header::CONTENT_TYPE, HeaderMap};
use pyo3::{prelude::*, types::PyAny, Py};

use crate::{cors::Cors, exceptions::*, json, status::Status, IntoPyException, Response};

type Error = Box<dyn std::error::Error>;

impl TryFrom<String> for Response {
    type Error = Error;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "text/plain".parse()?);
        Ok(Response {
            status: Status::OK,
            headers,
            body: val.clone().into(),
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
            body: json::dumps(&val)?.into(),
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
            body: val.0.clone().into(),
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
            body: json::dumps(&val.0)?.into(),
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
            body: Bytes::new(),
        }
    }
}

impl From<PyErr> for Response {
    fn from(value: PyErr) -> Self {
        Python::with_gil(|py| {
            let status = match value.is_instance_of::<BaseError>(py) {
                true if value.is_instance_of::<UnauthorizedError>(py) => Status::UNAUTHORIZED,
                true if value.is_instance_of::<ForbiddenError>(py) => Status::FORBIDDEN,
                true if value.is_instance_of::<NotFoundError>(py) => Status::NOT_FOUND,
                true if value.is_instance_of::<ConflictError>(py) => Status::CONFLICT,
                true if value.is_instance_of::<InternalError>(py) => Status::INTERNAL_SERVER_ERROR,
                true => Status::BAD_REQUEST,
                false => {
                    value.display(py);
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
