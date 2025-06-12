use crate::Status;
use pyo3::prelude::*;

/// A catcher for handling specific HTTP status codes.
///
/// Catchers allow you to provide custom responses for specific HTTP status codes.
/// They are typically created using the `catcher` decorator function.
///
/// Args:
///     status (Status): The HTTP status code this catcher will handle.
///     handler (callable): The handler function that will be called when this status occurs.
///
/// Example:
/// ```python
/// from oxapy import catcher, Status
///
/// @catcher(Status.NOT_FOUND)
/// def handle_not_found(request, response):
///     return Response("<h1>Custom 404 Page</h1>", content_type="text/html")
/// ```
#[pyclass]
pub struct Catcher {
    pub status: Status,
    pub handler: Py<PyAny>,
}

/// Internal builder class for creating catchers.
///
/// This class is returned by the `catcher` function and is used to create
/// a Catcher when called with a handler function.
#[pyclass]
pub struct CatcherBuilder {
    status: Status,
}

#[pymethods]
impl CatcherBuilder {
    /// Create a Catcher when called with a handler function.
    ///
    /// Args:
    ///     handler (callable): The handler function to call when the status occurs.
    ///
    /// Returns:
    ///     Catcher: A new catcher for the specified status.
    fn __call__(&self, handler: Py<PyAny>) -> Catcher {
        Catcher {
            status: self.status,
            handler,
        }
    }
}

/// Decorator for creating status code catchers.
///
/// A catcher allows you to provide custom responses for specific HTTP status codes.
///
/// Args:
///     status (Status): The HTTP status code to catch.
///
/// Returns:
///     CatcherBuilder: A builder that creates a Catcher when called with a handler function.
///
/// Example:
/// ```python
/// from oxapy import catcher, Status, Response
///
/// @catcher(Status.NOT_FOUND)
/// def handle_404(request, response):
///     return Response("<h1>Page Not Found</h1>", content_type="text/html")
///
/// # Add the catcher to your server
/// app.catchers([handle_404])
/// ```
#[pyfunction]
pub fn catcher(status: Status) -> CatcherBuilder {
    CatcherBuilder { status }
}
