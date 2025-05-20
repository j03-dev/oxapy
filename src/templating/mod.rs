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
    #[pyo3(signature=(dir="./templates/**/*.html", engine="jinja"))]
    fn new(dir: &str, engine: &str) -> PyResult<Template> {
        match engine {
            "jinja" => Ok(Template::Jinja(self::minijinja::Jinja::new(
                dir.to_string(),
            )?)),
            "tera" => Ok(Template::Tera(self::tera::Tera::new(dir.to_string())?)),
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
) -> PyResult<Response> {
    let template = request
        .template
        .as_ref()
        .ok_or_else(|| PyValueError::new_err("Not template"))?;

    let body = match template.as_ref() {
        Template::Jinja(engine) => engine.render(name, context)?,
        Template::Tera(engine) => engine.render(name, context)?,
    };

    Ok(Response {
        status: Status::OK,
        body: body.into(),
        headers: HashMap::from([("Content-Type".to_string(), "text/html".to_string())]),
    })
}

pub fn templating_submodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let templating = PyModule::new(m.py(), "templating")?;
    templating.add_function(wrap_pyfunction!(render, &templating)?)?;
    templating.add_class::<Template>()?;
    templating.add_class::<self::tera::Tera>()?;
    templating.add_class::<self::minijinja::Jinja>()?;
    m.add_submodule(&templating)
}
