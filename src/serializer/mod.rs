use pyo3::{
    prelude::*,
    types::{PyDict, PyType},
};
use serde_json::Value;

use crate::{request::Request, IntoPyException};

#[pyclass(subclass)]
#[derive(Debug, Clone)]
struct Field {
    #[pyo3(get)]
    required: Option<bool>,
    #[pyo3(get)]
    ty: String,
    #[pyo3(get)]
    format: Option<String>,
}

#[pymethods]
impl Field {
    #[new]
    #[pyo3(signature = (ty, required = true, format = None))]
    fn new(ty: String, required: Option<bool>, format: Option<String>) -> Self {
        Self {
            required,
            ty,
            format,
        }
    }
}

impl Field {
    fn to_json_schema_value(&self) -> Value {
        let mut schema = serde_json::Map::new();
        schema.insert("type".to_string(), Value::String(self.ty.clone()));
        if let Some(fmt) = &self.format {
            schema.insert("format".to_string(), Value::String(fmt.clone()));
        }
        Value::Object(schema)
    }
}

#[pyclass(subclass, extends=Field)]
#[derive(Debug)]
struct Serializer {
    #[pyo3(get, set)]
    validate_data: Option<Py<PyDict>>,
    #[pyo3(get, set)]
    request: Option<Request>,
}

#[pymethods]
impl Serializer {
    #[new]
    #[pyo3(signature = (request = None, required = true))]
    fn new(request: Option<Request>, required: Option<bool>) -> (Self, Field) {
        (
            Self {
                validate_data: None,
                request,
            },
            Field::new("object".to_string(), required, None),
        )
    }

    fn validate(mut slf: PyRefMut<'_, Serializer>, py: Python<'_>) -> PyResult<()> {
        let request = slf
            .request
            .as_ref()
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("No request provided"))?;

        let json_dict = request.body.clone().unwrap();
        let json_value: Value = serde_json::from_str(&json_dict.to_string()).into_py_exception()?;

        let py_dict = crate::json::loads(&json_value.to_string())?;

        slf.validate_data = Some(py_dict);

        let schema_value = Self::json_schema_value(py, &slf.into_pyobject(py)?.get_type())?;

        jsonschema::validate(&schema_value, &json_value).into_py_exception()?;

        Ok(())
    }
}

impl Serializer {
    fn json_schema_value(py: Python, cls: &Bound<'_, PyType>) -> PyResult<Value> {
        let mut properties = serde_json::Map::new();
        let mut required_fields = Vec::new();

        for attr in cls.dir()? {
            let attr_name = attr.to_string();
            if let Ok(attr_obj) = cls.getattr(&attr_name) {
                if let Ok(field) = attr_obj.extract::<PyRef<Field>>() {
                    properties.insert(attr_name.clone(), field.to_json_schema_value());
                    if field.required.unwrap_or(false) {
                        required_fields.push(attr_name);
                    }
                } else if let Ok(_nested_serializer) = attr_obj.extract::<PyRef<Serializer>>() {
                    let nested_schema = Self::json_schema_value(py, &attr_obj.get_type())?;
                    properties.insert(attr_name.clone(), nested_schema);
                    if let Ok(field) = attr_obj.extract::<PyRef<Field>>() {
                        if field.required.unwrap_or(false) {
                            required_fields.push(attr_name);
                        }
                    }
                }
            }
        }

        let mut schema = serde_json::Map::new();
        schema.insert("type".to_string(), Value::String("object".to_string()));
        schema.insert("properties".to_string(), Value::Object(properties));
        if !required_fields.is_empty() {
            let reqs: Vec<Value> = required_fields.into_iter().map(Value::String).collect();
            schema.insert("required".to_string(), Value::Array(reqs));
        }
        Ok(Value::Object(schema))
    }
}

#[pymodule]
pub fn serializer_submodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let serializer = PyModule::new(m.py(), "serializer")?;
    serializer.add_class::<Field>()?;
    serializer.add_class::<Serializer>()?;
    m.add_submodule(&serializer)?;
    Ok(())
}
