use hyper::{header::CONTENT_TYPE, HeaderMap};

use crate::{json, status::Status, Response};
use pyo3::{prelude::*, types::PyAny, Py};

impl From<String> for Response {
    fn from(val: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "text/plain".parse().unwrap());
        Response {
            status: Status::OK,
            headers,
            body: val.clone().into(),
        }
    }
}

impl From<PyObject> for Response {
    fn from(val: PyObject) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        Response {
            status: Status::OK,
            headers,
            body: json::dumps(&val).unwrap().into(),
        }
    }
}

impl From<(String, Status)> for Response {
    fn from(val: (String, Status)) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "text/plain".parse().unwrap());
        Response {
            status: val.1,
            headers,
            body: val.0.clone().into(),
        }
    }
}

impl From<(PyObject, Status)> for Response {
    fn from(val: (PyObject, Status)) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        Response {
            status: val.1,
            headers,
            body: json::dumps(&val.0).unwrap().into(),
        }
    }
}

macro_rules! to_response {
    ($rslt:expr, $py:expr, $($type:ty),*) => {{
        $(
            if let Ok(value) = $rslt.extract::<$type>($py) {
                return Ok(value.into());
            }
        )*

        return Err(pyo3::exceptions::PyException::new_err(
            "Failed to convert this type to response",
        ));
    }};
}

#[pyfunction]
pub fn convert_to_response(result: Py<PyAny>, py: Python<'_>) -> PyResult<Response> {
    to_response!(
        result,
        py,
        Response,
        Status,
        (String, Status),
        (PyObject, Status),
        String,
        PyObject
    )
}
