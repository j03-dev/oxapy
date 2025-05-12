use std::collections::HashMap;

use crate::{status::Status, Response};
use pyo3::{prelude::*, types::PyAny, Py};

impl Into<Response> for String {
    fn into(self) -> Response {
        Response {
            status: Status::OK,
            headers: HashMap::from([("Content-Type".to_string(), "text/plain".to_string())]),
            body: self.clone().into(),
        }
    }
}

impl Into<Response> for PyObject {
    fn into(self) -> Response {
        Response {
            status: Status::OK,
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
            body: crate::json::dumps(&self).unwrap().into(),
        }
    }
}

impl Into<Response> for (String, Status) {
    fn into(self) -> Response {
        Response {
            status: self.1.clone(),
            headers: HashMap::from([("Content-Type".to_string(), "text/plain".to_string())]),
            body: self.0.clone().into(),
        }
    }
}

impl Into<Response> for (PyObject, Status) {
    fn into(self) -> Response {
        Response {
            status: self.1.clone(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
            body: crate::json::dumps(&self.0).unwrap().into(),
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
