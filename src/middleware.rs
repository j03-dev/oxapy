use std::sync::Arc;

use pyo3::{Py, PyAny, PyResult, Python, call::PyCallArgs, prelude::*, types::PyDict};

#[derive(Clone, Debug)]
pub struct Middleware {
    handler: Arc<Py<PyAny>>,
    sequence: usize,
}

impl Middleware {
    pub fn new(handler: Py<PyAny>, sequence: usize) -> Self {
        Self {
            handler: Arc::new(handler),
            sequence,
        }
    }
}

pub struct MiddlewareChain;

impl MiddlewareChain {
    pub fn execute<'py, A>(
        py: Python<'py>,
        middlewares: &[Middleware],
        route_sequence: usize,
        route_handler: &Py<PyAny>,
        args: A,
        kwargs: Bound<'py, PyDict>,
    ) -> PyResult<Py<PyAny>>
    where
        A: PyCallArgs<'py>,
    {
        let handler =
            Self::build_middleware_chain(py, middlewares, route_sequence, route_handler, 0)?;
        handler.call(py, args, Some(&kwargs))
    }

    fn build_middleware_chain(
        py: Python<'_>,
        middlewares: &[Middleware],
        route_sequence: usize,
        route_handler: &Py<PyAny>,
        index: usize,
    ) -> PyResult<Py<PyAny>> {
        let Some(middleware) = middlewares
            .get(index)
            .filter(|m| m.sequence <= route_sequence)
        else {
            return Ok(route_handler.clone_ref(py));
        };
        let next = Self::build_middleware_chain(
            py,
            middlewares,
            route_sequence,
            route_handler,
            index + 1,
        )?;
        let globals = PyDict::new(py);
        globals.set_item("middleware", middleware.handler.clone_ref(py))?;
        globals.set_item("next", next)?;
        let wrapper = py.eval(
            c"lambda *args, **kwargs: middleware(next=next, *args, **kwargs)",
            Some(&globals),
            None,
        )?;
        Ok(wrapper.into())
    }
}
