use std::collections::HashMap;
use std::str;

use hyper::body::Bytes;
use pyo3::{prelude::*, types::PyBytes};

use crate::{
    into_response::IntoResponse,
    session::{Session, SessionStore},
    status::Status,
    IntoPyException,
};

#[derive(Clone)]
#[pyclass]
pub struct Response {
    #[pyo3(get, set)]
    pub status: Status,
    pub body: Bytes,
    #[pyo3(get, set)]
    pub headers: HashMap<String, String>,
}

#[pymethods]
impl Response {
    #[new]
    #[pyo3(signature=(status, body, content_type="application/json".to_string()))]
    pub fn new(
        status: PyRef<'_, Status>,
        body: PyObject,
        content_type: String,
        py: Python<'_>,
    ) -> PyResult<Self> {
        let body = if let Ok(bytes) = body.extract::<Py<PyBytes>>(py) {
            bytes.as_bytes(py).to_vec().into()
        } else if content_type == "application/json" {
            crate::json::dumps(&body)?.into()
        } else {
            body.to_string().into()
        };

        Ok(Self {
            status: status.clone(),
            body,
            headers: HashMap::from([("Content-Type".to_string(), content_type)]),
        })
    }

    #[getter]
    fn body(&self) -> PyResult<String> {
        Ok(str::from_utf8(&self.body).into_py_exception()?.to_string())
    }

    pub fn header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }
}

impl IntoResponse for Response {
    fn into_response(&self) -> PyResult<Response> {
        Ok(self.clone())
    }
}

impl Response {
    pub fn set_body(mut self, body: String) -> Self {
        self.body = body.into();
        self
    }

    pub fn set_session_cookie(&mut self, session: &Session, store: &SessionStore) {
        let cookie_header = store.get_cookie_header(session);
        self.headers.insert("Set-Cookie".to_string(), cookie_header);
    }
}

#[pyclass]
pub struct Redirect {
    #[pyo3(get, set)]
    location: String,
    #[pyo3(get, set)]
    status: Status,
}

#[pymethods]
impl Redirect {
    #[new]
    #[pyo3(signature = (location, status= None))]
    fn new(location: String, status: Option<Status>) -> Self {
        Self {
            location,
            status: status.unwrap_or(Status::MOVED_PERMANENTLY),
        }
    }
}

impl IntoResponse for Redirect {
    fn into_response(&self) -> PyResult<Response> {
        let mut response = self.status.into_response()?;
        response.header("Location".to_string(), self.location.clone());
        Ok(response)
    }
}
