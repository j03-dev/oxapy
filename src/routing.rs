use std::{collections::HashMap, sync::Arc};

use pyo3::{ffi::c_str, prelude::*, types::PyDict, Py, PyAny};

use crate::{middleware::Middleware, IntoPyException, MatchitRoute};

#[derive(Clone, Debug)]
#[pyclass]
pub struct Route {
    pub method: String,
    pub path: String,
    pub handler: Arc<Py<PyAny>>,
}

impl Default for Route {
    fn default() -> Self {
        Python::with_gil(|py| Self {
            method: "GET".to_string(),
            path: String::default(),
            handler: Arc::new(py.None()),
        })
    }
}

#[pymethods]
impl Route {
    #[new]
    #[pyo3(signature=(path, method=None))]
    pub fn new(path: String, method: Option<String>) -> Self {
        Route {
            method: method.unwrap_or("GET".to_string()),
            path,
            ..Default::default()
        }
    }

    fn __call__(&self, handler: Py<PyAny>) -> PyResult<Self> {
        Ok(Self {
            handler: Arc::new(handler),
            ..self.clone()
        })
    }

    fn __repr__(&self) -> String {
        format!("{:#?}", self.clone())
    }
}

macro_rules! method_decorator {
    ($($method:ident),*) => {
        $(
            #[pyfunction]
            #[pyo3(signature = (path, handler = None))]
            pub fn $method(path: String, handler: Option<Py<PyAny>>, py: Python<'_>) -> Route {
                Route {
                    method: stringify!($method).to_string().to_uppercase(),
                    path,
                    handler: Arc::new(handler.unwrap_or(py.None()))
                }
            }
        )+
    };
}

method_decorator!(get, post, put, patch, delete, head, options);

#[derive(Default, Clone, Debug)]
#[pyclass]
pub struct Router {
    pub routes: HashMap<String, matchit::Router<Route>>,
    pub middlewares: Vec<Middleware>,
}

#[pymethods]
impl Router {
    #[new]
    pub fn new() -> Self {
        Router::default()
    }

    fn middleware(&mut self, middleware: Py<PyAny>) {
        let middleware = Middleware::new(middleware);
        self.middlewares.push(middleware);
    }

    fn route(&mut self, route: Route) -> PyResult<()> {
        let method_router = self.routes.entry(route.method.clone()).or_default();
        method_router
            .insert(&route.path, route.clone())
            .into_py_exception()?;
        Ok(())
    }

    fn routes(&mut self, routes: Vec<Route>) -> PyResult<()> {
        for route in routes {
            self.route(route)?;
        }
        Ok(())
    }
}

impl Router {
    pub fn find<'l>(&'l self, method: &str, uri: &'l str) -> Option<MatchitRoute<'l>> {
        let path = uri.split('?').next().unwrap_or(uri);
        if let Some(router) = self.routes.get(method) {
            if let Ok(route) = router.at(path) {
                return Some(route);
            }
        }
        None
    }
}

#[pyfunction]
pub fn static_file(directory: String, path: String, py: Python<'_>) -> PyResult<Route> {
    let pathlib = py.import("pathlib")?;
    let oxapy = py.import("oxapy")?;
    let mimetypes = py.import("mimetypes")?;

    let globals = &PyDict::new(py);
    globals.set_item("Path", pathlib.getattr("Path")?)?;
    globals.set_item("directory", directory)?;
    globals.set_item("Status", oxapy.getattr("Status")?)?;
    globals.set_item("Response", oxapy.getattr("Response")?)?;
    globals.set_item("mimetypes", mimetypes)?;

    py.run(
        c_str!(
            r#"
def static_file(request, path):
    file_path = f"{directory}/{path}"
    try:
        with open(file_path, "rb") as f: content = f.read()
        content_type, _ = mimetypes.guess_type(file_path)
        return Response(Status.OK, content, content_type or "application/octet-stream")
    except FileNotFoundError:
        return Response(Status.NOT_FOUND, "File not found")
"#
        ),
        Some(globals),
        None,
    )?;

    let handler = globals.get_item("static_file")?.unwrap();

    let route = Route {
        path: format!("/{path}/{{*path}}"),
        handler: Arc::new(handler.into()),
        ..Default::default()
    };

    Ok(route)
}
