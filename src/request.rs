use std::sync::Arc;

use ahash::HashMap;

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

    /// Load the body as dictionary if body is json format
    ///
    /// Args: None
    ///
    /// Returns:
    ///     Return Dictionary: return body as dictionary from body
    //
    /// Raises:
    ///     Exception: if body is not present in the request
    ///
    /// Example:
    /// ```python
    /// data = request.json()
    /// value = data["key"]
    /// ```
    pub fn json(&self) -> PyResult<Py<PyDict>> {
        let data = self
            .body
            .as_ref()
            .ok_or_else(|| PyException::new_err("The body is not present"))?;
        json::loads(data)
    }

    /// Get app data from requeset
    ///
    /// Args: None
    ///
    /// Returns:
    ///     Return: Instance of `app_data` None if there is not app data in your app
    #[getter]
    fn app_data(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.app_data.as_ref().map(|d| d.clone_ref(py))
    }

    /// Get query from request uri
    ///
    /// Args: None
    ///
    /// Returns:
    ///     Return Dictionary: query from uri request and None if there is not query present
    ///
    /// Raises:
    ///     Exception: if the uri is not in right format
    ///
    /// Example:
    /// ```python
    /// # locahost:8000/api?key=value
    /// query = request.query()
    /// value = query["key"]
    /// ```
    fn query(&self) -> PyResult<Option<std::collections::HashMap<String, String>>> {
        let uri: Uri = self.uri.parse().into_py_exception()?;
        if let Some(query_string) = uri.query() {
            let parsed_query = form_urlencoded::parse(query_string.as_bytes())
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect();
            return Ok(Some(parsed_query));
        }
        Ok(None)
    }

    /// Get Session from requset
    ///
    /// Args: None
    ///
    /// Returns:
    ///     Return: Session instance
    ///
    /// Raises:
    ///     AttributeError: if session store is not configured on the app
    ///
    /// Example:
    /// ```python
    /// session = request.session()
    /// session["is_auth"] = true
    /// ```
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
