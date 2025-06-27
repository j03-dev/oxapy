use hyper::{
    header::{InvalidHeaderValue, CONTENT_TYPE},
    HeaderMap,
};

use crate::{json, status::Status, IntoPyException, Response};
use pyo3::{prelude::*, types::PyAny, Py};

impl TryFrom<String> for Response {
    type Error = InvalidHeaderValue;

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

impl TryFrom<PyObject> for Response {
    type Error = InvalidHeaderValue;

    fn try_from(val: PyObject) -> Result<Self, Self::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        Ok(Response {
            status: Status::OK,
            headers,
            body: json::dumps(&val).unwrap().into(),
        })
    }
}

impl TryFrom<(String, Status)> for Response {
    type Error = InvalidHeaderValue;

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

impl TryFrom<(PyObject, Status)> for Response {
    type Error = InvalidHeaderValue;

    fn try_from(val: (PyObject, Status)) -> Result<Self, Self::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        Ok(Response {
            status: val.1,
            headers,
            body: json::dumps(&val.0).unwrap().into(),
        })
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
