use std::sync::Arc;

use hyper::{HeaderMap, header::CONTENT_TYPE, http::HeaderValue};
use pyo3::{
    Bound, PyResult,
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyModule, PyModuleMethods},
};
use pyo3_stub_gen::derive::*;
use tera::{Function, TeraResult, Value};

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

impl Function<TeraResult<Value>> for PyTeraFunction {
    fn call(&self, kwargs: tera::Kwargs, state: &tera::State) -> TeraResult<Value> {
        let args: serde_json::Value = kwargs.deserialize()?;

        Python::attach(|py| {
            let py_kwargs = json::from_rstruct2pydict(args, py)
                .map_err(tera::Error::message)?
                .into_bound(py);
            let result = self
                .callable
                .call(py, (), Some(&py_kwargs))
                .map_err(tera::Error::message)?
                .into_bound(py);
            let json_value: serde_json::Value =
                json::from_pydict2rstruct(&result).map_err(tera::Error::message)?;
            Ok(tera::Value::from_serializable(&json_value))
        })
    }
}

/// Template engine for rendering HTML templates using Tera.
///
/// Templates are loaded lazily via `load()`. Custom functions must be registered
/// with `register_function()` before calling `load()`.
///
/// Args:
///     None
///
/// Returns:
///     Template: A new empty template engine instance.
///
/// Example:
/// ```python
/// from oxapy import templating
///
/// template = templating.Template()
/// template.register_function("_t", translate)
/// template.load("./templates/**/*.html")
/// result = template.render("index.html", {"title": "Hello"})
/// ```
#[gen_stub_pyclass]
#[pyclass(from_py_object, module = "oxapy.templating")]
#[derive(Clone, Debug)]
pub struct Template(Arc<tera::Tera>);

#[gen_stub_pymethods]
#[pymethods]
impl Template {
    /// Create a new empty Template instance.
    ///
    /// Templates are not loaded at construction time. Use `register_function()` to add
    /// custom functions, then `load()` to load and validate templates.
    ///
    /// Args:
    ///     None
    ///
    /// Returns:
    ///     Template: A new empty template engine instance.
    ///
    /// Example:
    /// ```python
    /// from oxapy import templating
    ///
    /// template = templating.Template()
    /// template.register_function("_t", translate)
    /// template.load("./templates/**/*.html")
    /// ```
    #[new]
    #[gen_stub(override_return_type(type_repr = "typing_extensions.Self", imports = ("typing_extensions",)))]
    pub fn new() -> PyResult<Self> {
        Ok(Self(Arc::new(tera::Tera::new())))
    }

    /// Load templates from a directory glob pattern.
    ///
    /// This parses and validates all matching template files. Any custom functions
    /// registered with `register_function()` must be added **before** calling `load()`,
    /// otherwise Tera will raise an error for unknown functions.
    ///
    /// Args:
    ///     dir (str, optional): Glob pattern to search for templates (default: "./templates/**/*.html").
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     RuntimeError: If the template engine is shared across multiple references.
    ///     PyException: If the glob pattern is invalid or templates contain errors.
    ///
    /// Example:
    /// ```python
    /// from oxapy import templating
    ///
    /// template = templating.Template()
    /// template.register_function("_t", translate)
    /// template.load("./templates/**/*.html")
    /// ```
    #[pyo3(signature=(dir="./templates/**/*.html"))]
    fn load(&mut self, dir: &str, py: Python<'_>) -> PyResult<()> {
        let callable = py.eval(
            c"lambda token: f'<input type=\"hidden\" name=\"_csrf_token\" value=\"{token}\">'",
            None,
            None,
        )?;
        self.register_function("csrf_input".to_string(), callable.into())?;

        if let Some(tera) = Arc::get_mut(&mut self.0) {
            tera.load_from_glob(dir).into_py_exception()?;
            Ok(())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Cannot load templates: the template engine is shared across multiple references. \
                 Create a new Template instance instead.",
            ))
        }
    }

    /// Register a Python function as a custom template function.
    ///
    /// This method allows you to expose Python callables to be used within Tera templates.
    /// The function will receive keyword arguments from the template call and should return
    /// a value that can be serialized to JSON.
    ///
    /// **Important:** All functions must be registered **before** calling `load()`.
    /// Tera validates function existence at template load time.
    ///
    /// Args:
    ///     name (str): The name used to call the function from templates (e.g., `{{ my_function(key=value) }}`).
    ///     callable (Callable): A Python callable that accepts keyword arguments and returns a value.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     RuntimeError: If called after `load()` has been invoked.
    ///
    /// Example:
    /// ```python
    /// template = templating.Template()
    /// template.register_function("add", lambda a, b: a + b)
    /// template.load("./templates/**/*.html")
    /// # In template: {{ add(a=1, b=2) }} -> 3
    /// ```
    pub fn register_function(&mut self, name: String, callable: Py<PyAny>) -> PyResult<()> {
        if let Some(tera) = Arc::get_mut(&mut self.0) {
            let py_func = PyTeraFunction { callable };
            tera.register_function(name, py_func);
            Ok(())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Cannot register function after templates have been loaded. \
                 Call register_function() before load().",
            ))
        }
    }
}

impl Template {
    pub fn render(
        &self,
        template_name: &str,
        context: Option<Bound<'_, PyDict>>,
    ) -> PyResult<String> {
        let mut ctx = tera::Context::new();
        if let Some(context) = context {
            let map: serde_json::Value = json::from_pydict2rstruct(&context)?;
            ctx = tera::Context::from_serialize(&map).into_py_exception()?;
        }
        self.0.render(template_name, &ctx).into_py_exception()
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

    if let Some(csrf_token) = request.ext.get("csrf_token") {
        ctx.set_item("csrf_token", csrf_token.clone_ref(py))?;
    }

    let body = template.render(name, Some(ctx))?;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
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
