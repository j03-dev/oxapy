use pyo3::prelude::*;
use pyo3::{create_exception, exceptions::PyException};

create_exception!(exceptions, BaseError, PyException);
create_exception!(exceptions, BadRequestError, BaseError);
create_exception!(exceptions, UnauthorizedError, BaseError);
create_exception!(exceptions, ForbiddenError, BaseError);
create_exception!(exceptions, NotFoundError, BaseError);
create_exception!(exceptions, ConflictError, BaseError);
create_exception!(exceptions, InternalError, BaseError);
create_exception!(exceptions, ValidationException, BaseError);

pub fn exceptions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let exceptions = PyModule::new(m.py(), "exceptions")?;
    exceptions.add("BaseError", m.py().get_type::<BaseError>())?;
    exceptions.add("BadRequestError", m.py().get_type::<BadRequestError>())?;
    exceptions.add("UnauthorizedError", m.py().get_type::<UnauthorizedError>())?;
    exceptions.add("ForebiddenError", m.py().get_type::<ForbiddenError>())?;
    exceptions.add("NotFoundError", m.py().get_type::<NotFoundError>())?;
    exceptions.add("ConflictError", m.py().get_type::<ConflictError>())?;
    exceptions.add("InternalError", m.py().get_type::<InternalError>())?;
    exceptions.add("ValidationException", m.py().get_type::<ValidationException>())?;
    m.add_submodule(&exceptions)
}
