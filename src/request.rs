use std::{collections::HashMap, sync::Arc};

use pyo3::{
    exceptions::{PyAttributeError, PyException},
    prelude::*,
    types::PyDict,
};

use hyper::Uri;
use url::form_urlencoded;

use crate::{
    json,
    multipart::File,
    session::{Session, SessionStore},
    templating::Template,
    IntoPyException,
};

#[derive(Clone, Debug, Default)]
#[pyclass]
pub struct Request {
    #[pyo3(get)]
    pub method: String,
    #[pyo3(get)]
    pub uri: String,
    #[pyo3(get)]
    pub headers: HashMap<String, String>,
    #[pyo3(get)]
    pub body: Option<String>,
    #[pyo3(get)]
    pub form: Option<HashMap<String, String>>,
    #[pyo3(get)]
    pub files: Option<HashMap<String, File>>,
    pub app_data: Option<Arc<Py<PyAny>>>,
    pub template: Option<Arc<Template>>,
    pub ext: HashMap<String, Arc<PyObject>>,
    pub session: Option<Arc<Session>>,
    pub session_store: Option<Arc<SessionStore>>,
}

#[pymethods]
impl Request {
    #[new]
    pub fn new(method: String, uri: String, headers: HashMap<String, String>) -> Self {
        Self {
            method,
            uri,
            headers,
            ..Default::default()
        }
    }

    pub fn json(&self) -> Option<Py<PyDict>> {
        self.body.as_ref().and_then(|data| json::loads(data).ok())
    }

    #[getter]
    fn app_data(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.app_data.as_ref().map(|d| d.clone_ref(py))
    }

    fn query(&self) -> PyResult<Option<HashMap<String, String>>> {
        let uri: Uri = self.uri.parse().into_py_exception()?;
        if let Some(query_string) = uri.query() {
            let parsed_query = form_urlencoded::parse(query_string.as_bytes())
                .filter_map(|(key, value)| Some((key.to_string(), value.to_string())))
                .collect();
            return Ok(Some(parsed_query));
        }
        Ok(None)
    }

    pub fn session(&self) -> PyResult<Session> {
        let message = "Session not available. Make sure you've configured SessionStore.";
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| PyAttributeError::new_err(message))?;
        Ok(session.as_ref().clone())
    }

    fn __getattr__(&self, py: Python<'_>, name: &str) -> PyResult<PyObject> {
        let message = format!("Request object has no attribute {name}");
        let obj = self
            .ext
            .get(name)
            .ok_or_else(|| PyAttributeError::new_err(message))?;
        Ok(obj.clone_ref(py))
    }

    fn __setattr__(&mut self, name: &str, value: PyObject) -> PyResult<()> {
        match name {
            "method" | "uri" | "headers" | "body" | "template" => Err(PyException::new_err(
                format!("Attribute '{}' is read-only and cannot be set", name),
            )),
            _ => {
                self.ext.insert(name.to_string(), Arc::new(value));
                Ok(())
            }
        }
    }

    pub fn __repr__(&self) -> String {
        format!("{:#?}", self)
    }
}
