use std::sync::Arc;

use pyo3::prelude::*;

use crate::status::Status;

#[derive(Clone)]
#[pyclass]
pub struct Catcher {
    pub status: Status,
    pub handler: Arc<Py<PyAny>>,
}

#[pymethods]
impl Catcher {
    #[new]
    pub fn new(status: PyRef<'_, Status>, py: Python<'_>) -> Self {
        Self {
            status: status.clone(),
            handler: Arc::new(py.None()),
        }
    }

    fn __call__(&self, handler: Py<PyAny>) -> PyResult<Self> {
        Ok(Self {
            handler: Arc::new(handler),
            ..self.clone()
        })
    }
}

#[pyfunction]
pub fn catcher(status: PyRef<'_, Status>, py: Python<'_>) -> Catcher {
    Catcher::new(status, py)
}
