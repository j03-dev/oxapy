use std::sync::Arc;

use pyo3::{
    prelude::*,
    types::{PyDict, PyType},
};

use crate::request::Request;

#[pyclass(subclass)]
struct Field {
    required: Option<bool>,
    ty: String,
    format: Option<String>,
}

#[pymethods]
impl Field {
    #[new]
    #[pyo3(signature=(ty, required=false, format = None))]
    fn new(ty: String, required: Option<bool>, format: Option<String>) -> Self {
        Self {
            required,
            ty,
            format,
        }
    }
}

#[pyclass(subclass, extends=Field)]
struct Serializer {
    validate_data: Option<Arc<Py<PyDict>>>,
    request: Option<Request>,
}

#[pymethods]
impl Serializer {
    #[new]
    #[pyo3(signature=(request=None, required=false))]
    fn new(request: Option<Request>, required: Option<bool>) -> (Self, Field) {
        (
            Self {
                validate_data: None,
                request,
            },
            Field::new("object".to_string(), required, None),
        )
    }

    #[classmethod]
    fn valide(cls: &Bound<'_, PyType>) -> PyResult<()> {
        for f in cls.dir()? {
            let field_name = f.to_string();
            let field = cls.getattr(field_name.clone())?;
            if let Ok(field_serializer) = field.extract::<PyRef<Field>>() {
                todo!()
            }
        }

        Ok(())
    }
}

pub fn serializer_submodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let serializer = PyModule::new(m.py(), "serializer")?;
    serializer.add_class::<Field>()?;
    serializer.add_class::<Serializer>()?;
    m.add_submodule(&serializer)
}
