use std::{collections::HashMap, sync::Arc};

use pyo3::{prelude::*, types::PyDict, IntoPyObjectExt};

use crate::IntoPyException;

#[derive(Debug, Clone)]
#[pyclass]
pub struct Tera {
    engine: Arc<tera::Tera>,
}

#[pymethods]
impl Tera {
    #[new]
    pub fn new(dir: String) -> PyResult<Self> {
        Ok(Self {
            engine: Arc::new(tera::Tera::new(&dir).into_py_exception()?),
        })
    }

    #[pyo3(signature=(template_name, context=None))]
    pub fn render(
        &self,
        template_name: String,
        context: Option<Bound<'_, PyDict>>,
        py: Python<'_>,
    ) -> PyResult<String> {
        let mut tera_context = tera::Context::new();

        if let Some(context) = context {
            let serialize = crate::json::dumps(&context.into_py_any(py)?)?;
            let map: HashMap<String, serde_json::Value> =
                serde_json::from_str(&serialize).into_py_exception()?;
            for (key, value) in map {
                tera_context.insert(key, &value);
            }
        }

        self.engine
            .render(&template_name, &tera_context)
            .into_py_exception()
    }
}
