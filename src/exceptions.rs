use pyo3::exceptions::PyException;
use pyo3::{impl_exception_boilerplate, prelude::*};
use pyo3_stub_gen::derive::*;

pub trait IntoPyException<T> {
    fn into_py_exception(self) -> PyResult<T>;
}

impl<T, E: ToString> IntoPyException<T> for Result<T, E> {
    fn into_py_exception(self) -> PyResult<T> {
        self.map_err(|err| PyException::new_err(err.to_string()))
    }
}

macro_rules! extend_exception {
    ($name:ident, $extend:ident) => {
        #[pyo3_stub_gen::derive::gen_stub_pymethods]
        #[pyo3::prelude::pymethods]
        impl $name {
            #[new]
            fn new(e: pyo3::Py<pyo3::PyAny>) -> ($name, $extend) {
                ($name, $extend(e))
            }
        }
    };
}

/// Base exception for all client-related HTTP errors.
///
/// This is the parent class for all HTTP client error exceptions (4xx status codes).
/// It extends Python's built-in Exception class and serves as a base for more
/// specific HTTP error types.
///
/// This exception is typically not raised directly, but rather one of its
/// more specific subclasses should be used.
#[gen_stub_pyclass]
#[pyclass(subclass, extends=PyException, module="oxapy.exceptions")]
pub struct ClientError(pub Py<PyAny>);

impl_exception_boilerplate!(ClientError);

#[gen_stub_pymethods]
#[pymethods]
impl ClientError {
    #[new]
    fn new(e: Py<PyAny>) -> ClientError {
        Self(e)
    }
}

/// HTTP 400 Bad Request error exception.
///
/// Raised when the server cannot process the request due to client error,
/// such as malformed request syntax, invalid request message framing,
/// or deceptive request routing.
///
/// This exception corresponds to HTTP status code 400.
#[gen_stub_pyclass]
#[pyclass(extends=ClientError, module="oxapy.exceptions")]
pub struct BadRequestError;

impl_exception_boilerplate!(BadRequestError);
extend_exception!(BadRequestError, ClientError);

/// HTTP 401 Unauthorized error exception.
///
/// Raised when authentication is required and has failed or has not been provided.
/// The client must authenticate itself to get the requested response.
///
/// This exception corresponds to HTTP status code 401.
#[gen_stub_pyclass]
#[pyclass(extends=ClientError, module="oxapy.exceptions")]
pub struct UnauthorizedError;

impl_exception_boilerplate!(UnauthorizedError);
extend_exception!(UnauthorizedError, ClientError);

/// HTTP 403 Forbidden error exception.
///
/// Raised when the client is authenticated but does not have permission
/// to access the requested resource. Unlike 401 Unauthorized, the client's
/// identity is known to the server but lacks sufficient privileges.
///
/// This exception corresponds to HTTP status code 403.
#[gen_stub_pyclass]
#[pyclass(extends=ClientError, module="oxapy.exceptions")]
pub struct ForbiddenError;

impl_exception_boilerplate!(ForbiddenError);
extend_exception!(ForbiddenError, ClientError);

/// HTTP 404 Not Found error exception.
///
/// Raised when the requested resource could not be found on the server.
/// This is one of the most common HTTP errors and indicates that the
/// server has not found anything matching the requested URI.
///
/// This exception corresponds to HTTP status code 404.
#[gen_stub_pyclass]
#[pyclass(extends=ClientError, module="oxapy.exceptions")]
pub struct NotFoundError;

impl_exception_boilerplate!(NotFoundError);
extend_exception!(NotFoundError, ClientError);

/// HTTP 409 Conflict error exception.
///
/// Raised when the request could not be completed due to a conflict with
/// the current state of the resource. This often occurs in scenarios involving
/// resource creation where a duplicate would be created, or during updates
/// that would create an inconsistent state.
///
/// This exception corresponds to HTTP status code 409.
#[gen_stub_pyclass]
#[pyclass(extends=ClientError, module="oxapy.exceptions")]
pub struct ConflictError;

impl_exception_boilerplate!(ConflictError);
extend_exception!(ConflictError, ClientError);

/// HTTP 500 Internal Server Error exception.
///
/// Raised when the server encounters an unexpected condition that prevents
/// it from fulfilling the request. This is a generic error message when
/// no more specific message is suitable.
///
/// This exception corresponds to HTTP status code 500.
#[gen_stub_pyclass]
#[pyclass(extends=PyException, module="oxapy.exceptions")]
#[repr(transparent)]
pub struct InternalError(Py<PyAny>);

impl_exception_boilerplate!(InternalError);

#[gen_stub_pymethods]
#[pymethods]
impl InternalError {
    #[new]
    fn new(e: Py<PyAny>) -> InternalError {
        Self(e)
    }
}

pub fn exceptions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let exceptions = PyModule::new(m.py(), "exceptions")?;
    exceptions.add("ClientError", m.py().get_type::<ClientError>())?;
    exceptions.add("BadRequestError", m.py().get_type::<BadRequestError>())?;
    exceptions.add("UnauthorizedError", m.py().get_type::<UnauthorizedError>())?;
    exceptions.add("ForbiddenError", m.py().get_type::<ForbiddenError>())?;
    exceptions.add("NotFoundError", m.py().get_type::<NotFoundError>())?;
    exceptions.add("ConflictError", m.py().get_type::<ConflictError>())?;
    exceptions.add("InternalError", m.py().get_type::<InternalError>())?;
    m.add_submodule(&exceptions)
}
