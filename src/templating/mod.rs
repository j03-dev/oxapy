use std::collections::HashMap;

use pyo3::{
    exceptions::{PyException, PyValueError},
    prelude::*,
    types::{PyDict, PyModule, PyModuleMethods},
    Bound, PyResult,
};

use crate::{request::Request, response::Response, status::Status};

mod minijinja;
mod tera;

#[derive(Clone, Debug)]
#[pyclass]
pub enum Template {
    Jinja(self::minijinja::Jinja),
    Tera(self::tera::Tera),
}

#[pymethods]
impl Template {
    #[new]
    fn new(engine: String, dir: String) -> PyResult<Template> {
        match engine.as_str() {
            "jinja" => Ok(Template::Jinja(self::minijinja::Jinja::new(dir)?)),
            "tera" => Ok(Template::Tera(self::tera::Tera::new(dir)?)),
            e => Err(PyException::new_err(format!(
                "Invalid engine type '{e}'. Valid options are 'jinja' or 'tera'.",
            ))),
        }
    }
}

#[pyfunction]
#[pyo3(signature=(request, name, context=None))]
fn render(
    request: Request,
    name: String,
    context: Option<Bound<'_, PyDict>>,
    py: Python<'_>,
) -> PyResult<Response> {
    let template = request
        .template
        .as_ref()
        .ok_or_else(|| PyValueError::new_err("Not template"))?;

    let body = match template.as_ref() {
        Template::Jinja(engine) => engine.render(name, context, py)?,
        Template::Tera(engine) => engine.render(name, context, py)?,
    };

    Ok(Response {
        status: Status::OK,
        body: body.into(),
        headers: HashMap::from([("Content-Type".to_string(), "text/html".to_string())]),
    })
}

pub fn templating_submodule(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let templating = PyModule::new(parent_module.py(), "templating")?;
    templating.add_function(wrap_pyfunction!(render, &templating)?)?;
    templating.add_class::<Template>()?;
    templating.add_class::<self::tera::Tera>()?;
    templating.add_class::<self::minijinja::Jinja>()?;
    parent_module.add_submodule(&templating)
}
