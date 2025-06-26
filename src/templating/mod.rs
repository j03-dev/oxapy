use hyper::{header::CONTENT_TYPE, HeaderMap};
use pyo3::{
    exceptions::{PyException, PyValueError},
    prelude::*,
    types::{PyDict, PyModule, PyModuleMethods},
    Bound, PyResult,
};

use crate::{request::Request, response::Response, status::Status};

mod minijinja;
mod tera;

/// Template engine for rendering HTML templates.
///
/// This class provides a unified interface for different template engines,
/// currently supporting both Jinja and Tera templates.
///
/// Args:
///     dir (str, optional): Directory pattern to search for templates (default: "./templates/**/*.html").
///     engine (str, optional): Template engine to use, either "jinja" or "tera" (default: "jinja").
///
/// Returns:
///     Template: A new template engine instance.
///
/// Raises:
///     PyException: If an invalid engine type is specified.
///
/// Example:
/// ```python
/// from oxapy import HttpServer
/// from oxapy.templating import Template
///
/// app = HttpServer(("127.0.0.1", 8000))
///
/// # Configure templates with default settings (Jinja)
/// app.template(Template())
///
/// # Or use Tera with custom template directory
/// app.template(Template("./views/**/*.html", "tera"))
/// ```
#[derive(Clone, Debug)]
#[pyclass]
pub enum Template {
    Jinja(self::minijinja::Jinja),
    Tera(self::tera::Tera),
}

#[pymethods]
impl Template {
    /// Create a new Template instance.
    ///
    /// Args:
    ///     dir (str, optional): Directory pattern to search for templates (default: "./templates/**/*.html").
    ///     engine (str, optional): Template engine to use, either "jinja" or "tera" (default: "jinja").
    ///
    /// Returns:
    ///     Template: A new template engine instance.
    ///
    /// Raises:
    ///     PyException: If an invalid engine type is specified.
    ///
    /// Example:
    /// ```python
    /// # Use Jinja with default template directory
    /// template = Template()
    ///
    /// # Use Tera with custom template directory
    /// template = Template("./views/**/*.html", "tera")
    /// ```
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

/// Render a template and return the result as an HTTP response.
///
/// This function renders a template using the template engine configured for the request.
///
/// Args:
///     request (Request): The HTTP request object containing template configuration.
///     name (str): The name of the template to render.
///     context (dict, optional): Template variables to use during rendering.
///
/// Returns:
///     Response: An HTTP response with the rendered template as HTML.
///
/// Raises:
///     PyValueError: If no template engine is configured for the request.
///
/// Example:
/// ```python
/// from oxapy import Router
/// from oxapy import templating
///
/// router = Router()
///
/// @router.get("/")
/// def index(request):
///     return templating.render(request, "index.html", {"title": "Home Page"})
/// ```
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

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "text/html".parse().unwrap());
    Ok(Response {
        status: Status::OK,
        body: body.into(),
        headers,
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
