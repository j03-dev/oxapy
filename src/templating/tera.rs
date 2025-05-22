use std::{collections::HashMap, sync::Arc};

use pyo3::{prelude::*, types::PyDict};

use crate::{IntoPyException, Wrap};

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
    ) -> PyResult<String> {
        let mut tera_context = tera::Context::new();
        if let Some(context) = context {
            let map: Wrap<HashMap<String, serde_json::Value>> = context.into();
            for (key, value) in map.0 {
                tera_context.insert(key, &value);
            }
        }

        self.engine
            .render(&template_name, &tera_context)
            .into_py_exception()
    }
}
