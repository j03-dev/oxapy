use crate::Status;
use pyo3::prelude::*;

#[pyclass]
pub struct Catcher {
    pub status: Status,
    pub handler: Py<PyAny>,
}

#[pyclass]
pub struct CatcherBuilder {
    status: Status,
}

#[pymethods]
impl CatcherBuilder {
    fn __call__(&self, handler: Py<PyAny>) -> Catcher {
        Catcher {
            status: self.status,
            handler,
        }
    }
}

#[pyfunction]
pub fn catcher(status: Status) -> CatcherBuilder {
    CatcherBuilder { status }
}
