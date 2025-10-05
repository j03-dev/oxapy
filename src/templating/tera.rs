use std::sync::Arc;

use crate::json;
use crate::IntoPyException;
use ahash::HashMap;
use pyo3::{prelude::*, types::PyDict};
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(module = "oxapy.templating")]
#[derive(Debug, Clone)]
pub struct Tera {
    engine: Arc<tera::Tera>,
}

#[gen_stub_pymethods]
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
    ) -> PyResult<String> {
        let mut tera_context = tera::Context::new();
        if let Some(context) = context {
            let map: json::Wrap<HashMap<String, serde_json::Value>> = context.try_into()?;
            for (key, value) in map.0 {
                tera_context.insert(key, &value);
            }
        }

        self.engine
            .render(&template_name, &tera_context)
            .into_py_exception()
    }
}
