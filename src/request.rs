use std::{collections::HashMap, sync::Arc};

use pyo3::{
    exceptions::{PyAttributeError, PyException, PyValueError},
    prelude::*,
    types::PyDict,
};

use crate::{
    multipart::File,
    session::{Session, SessionStore},
    templating::Template,
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

    pub fn json(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        if let Some(ref body) = self.body {
            crate::json::loads(body)
        } else {
            Ok(PyDict::new(py).into())
        }
    }

    #[getter]
    fn app_data(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.app_data.as_ref().map(|d| d.clone_ref(py))
    }

    fn query(&self, py: Python<'_>) -> PyResult<Option<HashMap<String, String>>> {
        let query_string = self.uri.split('?').nth(1);
        if let Some(query) = query_string {
            let query_params = Self::parse_query_string(query, py)?;
            return Ok(Some(query_params));
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

impl Request {
    fn parse_query_string(query_string: &str, py: Python<'_>) -> PyResult<HashMap<String, String>> {
        let urllib = PyModule::import(py, "urllib")?;
        let unqoute = urllib.getattr("parse")?.getattr("unquote")?;
        let mut result = HashMap::new();

        for pair in query_string.split("&") {
            let mut parts = pair.split("=");
            let key = parts
                .next()
                .ok_or_else(|| PyValueError::new_err("Invalide query string format"))?
                .to_string();
            let value = parts.next().unwrap_or_default();
            let value_parsed: String = unqoute.call1((value,))?.extract()?;
            result.insert(key, value_parsed);
        }

        Ok(result)
    }
}
