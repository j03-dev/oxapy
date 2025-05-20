use std::{collections::HashMap, sync::Arc};

use minijinja::Environment;
use pyo3::{prelude::*, types::PyDict};

use crate::{IntoPyException, Wrap};

#[derive(Debug, Clone)]
#[pyclass]
pub struct Jinja {
    engine: Arc<Environment<'static>>,
}

#[pymethods]
impl Jinja {
    #[new]
    pub fn new(dir: String) -> PyResult<Self> {
        let mut env = Environment::new();

        let paths = glob::glob(&dir).into_py_exception()?;

        for entry in paths {
            let path = entry.into_py_exception()?;
            if path.is_file() {
                let name = {
                    let full_path = path.to_str().unwrap().to_string();
                    let name = full_path.split("/").skip(1);
                    name.collect::<Vec<_>>().join("/")
                };
                let content = std::fs::read_to_string(&path)?;
                let name = Box::leak(name.into_boxed_str());
                let content = Box::leak(content.into_boxed_str());
                env.add_template(name, content).into_py_exception()?;
            }
        }

        Ok(Self {
            engine: Arc::new(env),
        })
    }

    #[pyo3(signature=(template_name, context=None))]
    pub fn render(
        &self,
        template_name: String,
        context: Option<Bound<'_, PyDict>>,
    ) -> PyResult<String> {
        let template = self
            .engine
            .get_template(&template_name)
            .into_py_exception()?;
        let mut ctx_values: HashMap<String, serde_json::Value> = HashMap::new();
        if let Some(context) = context {
            let Wrap::<_>(value) = context.into();
            ctx_values = value;
        }
        template.render(ctx_values).into_py_exception()
    }
}
