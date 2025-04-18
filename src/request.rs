use std::{collections::HashMap, sync::Arc};

use pyo3::{exceptions::PyAttributeError, prelude::*, types::PyDict};

use crate::templating::Template;

#[derive(Clone, Debug)]
#[pyclass]
pub struct Request {
    #[pyo3(get, set)]
    pub method: String,
    #[pyo3(get, set)]
    pub uri: String,
    #[pyo3(get, set)]
    pub headers: HashMap<String, String>,
    #[pyo3(get, set)]
    pub body: Option<String>,
    pub app_data: Option<Arc<Py<PyAny>>>,
    pub template: Option<Arc<Template>>,
    pub ext: HashMap<String, Arc<PyObject>>,
    pub form_data: Option<HashMap<String, String>>,
    pub files: Option<HashMap<String, crate::multipart::File>>,
}

#[pymethods]
impl Request {
    #[new]
    pub fn new(method: String, uri: String, headers: HashMap<String, String>) -> Self {
        Self {
            method,
            uri,
            headers,
            app_data: None,
            body: None,
            template: None,
            ext: HashMap::new(),
            form_data: None,
            files: None,
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

    fn query(&self) -> PyResult<Option<HashMap<String, String>>> {
        let query_string = self.uri.split('?').nth(1);
        if let Some(query) = query_string {
            let query_params = Self::parse_query_string(query.to_string());
            return Ok(Some(query_params));
        }
        Ok(None)
    }

    pub fn form(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        if let Some(ref form_data) = self.form_data {
            let dict = PyDict::new(py);
            for (key, value) in form_data {
                dict.set_item(key, value)?;
            }
            Ok(dict.into())
        } else {
            Ok(PyDict::new(py).into())
        }
    }

    pub fn files(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        if let Some(ref files) = self.files {
            let dict = PyDict::new(py);
            for (key, file) in files {
                dict.set_item(key, file.clone())?; // Assuming File is PyClass
            }
            Ok(dict.into())
        } else {
            Ok(PyDict::new(py).into())
        }
    }

    fn __getattr__(&self, py: Python<'_>, name: &str) -> PyResult<PyObject> {
        if let Some(value) = self.ext.get(name) {
            Ok(value.clone_ref(py))
        } else {
            Err(PyAttributeError::new_err(format!(
                "Request object has no attribute {name}"
            )))
        }
    }

    fn __setattr__(&mut self, name: &str, value: PyObject) -> PyResult<()> {
        match name {
            "method" | "uri" | "headers" | "body" | "template" => Ok(()),
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
    fn parse_query_string(query_string: String) -> HashMap<String, String> {
        query_string
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.split('=');
                let key = parts.next()?.to_string();
                let value = parts.next().map_or("".to_string(), |v| v.to_string());
                Some((key, value))
            })
            .collect()
    }
}
