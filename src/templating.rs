use std::sync::Arc;

use ahash::HashMap;
use hyper::{HeaderMap, header::CONTENT_TYPE};
use pyo3::{
    Bound, PyResult,
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyModule, PyModuleMethods},
};
use pyo3_stub_gen::derive::*;
use tera::{Function, Result as TeraResult, Value};

use crate::{
    exceptions::IntoPyException,
    json,
    request::Request,
    response::{Response, ResponseBody},
    status::Status,
};

struct PyTeraFunction {
    callable: Py<PyAny>,
}

impl Function for PyTeraFunction {
    fn call(&self, args: &std::collections::HashMap<String, Value>) -> TeraResult<Value> {
        Python::attach(|py| {
            let py_kwargs = json::from_rstruct2pydict(args, py)
                .map_err(tera::Error::msg)?
                .into_bound(py);
            let result = self
                .callable
                .call(py, (), Some(&py_kwargs))
                .map_err(tera::Error::msg)?
                .into_bound(py);
            json::from_pydict2rstruct(&result).map_err(tera::Error::msg)
        })
    }

    fn is_safe(&self) -> bool {
        true
    }
}

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
/// from oxapy import HttpServer, templating
///
/// app = HttpServer(("127.0.0.1", 8000))
///
/// # Configure templates with default settings (Jinja)
/// app.template(templating.Template())
///
/// # Or use Tera with custom template directory
/// app.template(templating.Template("./views/**/*.html", "tera"))
/// ```
#[pyclass(from_py_object, module = "oxapy.templating")]
#[gen_stub_pyclass]
#[derive(Clone, Debug)]
pub struct Template {
    engine: Arc<tera::Tera>,
}

#[gen_stub_pymethods]
#[pymethods]
impl Template {
    /// Create a new Template instance.
    ///
    /// Args:
    ///     dir (str, optional): Directory pattern to search for templates (default: "./templates/**/*.html").
    ///
    /// Returns:
    ///     Template: A new template engine instance.
    ///
    /// Raises:
    ///     PyException: If an invalid engine type is specified.
    ///
    /// Example:
    /// ```python
    /// from oxapy import templating
    ///
    /// # Use Jinja with default template directory
    /// template = templating.Template()
    ///
    /// # Use Tera with custom template directory
    /// template = templating.Template("./views/**/*.html")
    /// ```
    #[new]
    #[pyo3(signature=(dir="./templates/**/*.html"))]
    #[gen_stub(override_return_type(type_repr = "typing_extensions.Self", imports = ("typing_extensions",)))]
    pub fn new(dir: &str) -> PyResult<Self> {
        let tera = tera::Tera::new(dir).into_py_exception()?;
        Ok(Self {
            engine: Arc::new(tera),
        })
    }

    #[pyo3(signature=(template_name, context=None))]
    pub fn render(
        &self,
        template_name: &str,
        context: Option<Bound<'_, PyDict>>,
    ) -> PyResult<String> {
        let mut tera_context = tera::Context::new();
        if let Some(context) = context {
            let map: HashMap<String, serde_json::Value> = json::from_pydict2rstruct(&context)?;
            for (key, value) in map {
                tera_context.insert(key, &value);
            }
        }

        self.engine
            .render(template_name, &tera_context)
            .into_py_exception()
    }

    /// Register a Python function as a custom template function.
    ///
    /// This method allows you to expose Python callables to be used within Tera templates.
    /// The function will receive keyword arguments from the template call and should return
    /// a value that can be serialized to JSON.
    ///
    /// Args:
    ///     name (str): The name used to call the function from templates (e.g., `{{ my_function(key=value) }}`).
    ///     callable (Callable): A Python callable that accepts keyword arguments and returns a value.
    ///
    /// Returns:
    ///     None: This method does not return a value.
    ///
    /// Raises:
    ///     RuntimeError: If the template engine has been cloned and is shared across multiple references.
    ///
    /// Example:
    /// ```python
    /// template.register_function("add", lambda a, b: a + b)
    /// # In template: {{ add(a=1, b=2) }} -> 3
    /// ```
    pub fn register_function(&mut self, name: &str, callable: Py<PyAny>) -> PyResult<()> {
        if let Some(tera) = Arc::get_mut(&mut self.engine) {
            let py_func = PyTeraFunction { callable };
            tera.register_function(name, py_func);
            Ok(())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Cannot register function: Tera engine is already shared cloned copies",
            ))
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
/// from oxapy import Router, get, render
///
/// router = Router()
///
/// @get("/")
/// def index(request):
///     return render(request, "index.html", {"title": "Home Page"})
/// ```
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature=(request, name, context=None))]
fn render(
    request: Request,
    name: &str,
    context: Option<Bound<'_, PyDict>>,
    py: Python<'_>,
) -> PyResult<Response> {
    let template = request
        .template
        .as_ref()
        .ok_or_else(|| PyValueError::new_err("Not template"))?;

    let ctx = context.unwrap_or(PyDict::new(py));

    if let Some(session) = request.ext.get("session") {
        ctx.set_item("session", session.clone_ref(py))?;
    }

    let body = template.render(name, Some(ctx))?;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "text/html".parse().unwrap());
    Ok(Response {
        status: Status::OK,
        body: ResponseBody::Bytes(body.into()),
        headers,
    })
}

pub fn templating_submodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let templating = PyModule::new(m.py(), "templating")?;
    templating.add_class::<Template>()?;
    m.add_function(wrap_pyfunction!(render, m)?)?;
    m.add_submodule(&templating)
}
