use std::collections::HashMap;
use std::str;

use hyper::body::Bytes;
use pyo3::{prelude::*, types::PyBytes};

use crate::{
    session::{Session, SessionStore},
    status::Status,
    IntoPyException,
};

#[derive(Clone)]
#[pyclass(subclass)]
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
        status: Status,
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
            status,
            body,
            headers: HashMap::from([("Content-Type".to_string(), content_type)]),
        })
    }

    #[getter]
    fn body(&self) -> PyResult<String> {
        Ok(str::from_utf8(&self.body).into_py_exception()?.to_string())
    }

    pub fn header(&mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self.clone()
    }

    pub fn status(&mut self, status: Status) -> Self {
        self.status = status;
        self.clone()
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

#[pyclass(subclass, extends=Response)]
pub struct Redirect;

#[pymethods]
impl Redirect {
    #[new]
    fn new(location: String) -> (Self, Response) {
        (
            Self,
            Response {
                status: Status::MOVED_PERMANENTLY,
                body: Bytes::new(),
                headers: HashMap::from([
                    ("Content-Type".to_string(), "text/html".to_string()),
                    ("Location".to_string(), location.to_string()),
                ]),
            },
        )
    }
}
