use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyList, PyType},
    IntoPyObjectExt,
};
use serde_json::Value;

use crate::{request::Request, IntoPyException};

use fields::{
    BooleanField, CharField, DateField, DateTimeField, EmailField, EnumField, Field, IntegerField,
    NumberField, UUIDField,
};

mod fields;

#[pyclass(subclass, extends=Field)]
#[derive(Debug)]
struct Serializer {
    #[pyo3(get, set)]
    instance: Option<Py<PyAny>>,
    #[pyo3(get, set)]
    validate_data: Option<Py<PyDict>>,
    #[pyo3(get, set)]
    request: Option<Request>,
}

#[pymethods]
impl Serializer {
    #[new]
    #[pyo3(signature = (
        request = None,
        instance = None,
        required = true,
        many = false,
        title = None,
        description = None
    ))]
    fn new(
        request: Option<Request>,
        instance: Option<Py<PyAny>>,
        required: Option<bool>,
        many: Option<bool>,
        title: Option<String>,
        description: Option<String>,
    ) -> (Self, Field) {
        (
            Self {
                validate_data: None,
                instance,
                request,
            },
            Field::new(
                "object".to_string(),
                required,
                None,
                many,
                None,
                None,
                None,
                None,
                None,
                None,
                title,
                description,
            ),
        )
    }

    fn schema(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let schema_value = Self::json_schema_value(&slf.into_pyobject(py)?.get_type())?;
        crate::json::loads(&schema_value.to_string())
    }

    fn validate(mut slf: PyRefMut<'_, Self>, py: Python<'_>) -> PyResult<()> {
        let request = slf
            .request
            .as_ref()
            .ok_or_else(|| PyValueError::new_err("No request provided"))?;

        let json_dict = request
            .body
            .clone()
            .ok_or_else(|| PyValueError::new_err("Request body is empty"))?;

        let json_value: Value = serde_json::from_str(&json_dict.to_string()).into_py_exception()?;

        let py_dict = crate::json::loads(&json_value.to_string())?;

        slf.validate_data = Some(py_dict);

        let schema_value = Self::json_schema_value(&slf.into_pyobject(py)?.get_type())?;

        let validator = jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)
            .into_py_exception()?;

        validator.validate(&json_value).into_py_exception()?;

        Ok(())
    }

    fn to_representation<'l>(
        slf: Bound<'_, Self>,
        instance: Bound<PyAny>,
        py: Python<'l>,
    ) -> PyResult<Bound<'l, PyDict>> {
        let dict = PyDict::new(py);
        let columns = instance
            .getattr("__table__")?
            .getattr("columns")?
            .try_iter()?;
        for c in columns {
            let col = c.unwrap().getattr("name")?.to_string();
            if slf.getattr(&col).is_ok() {
                dict.set_item(&col, instance.getattr(&col)?)?;
            }
        }
        Ok(dict)
    }

    #[getter]
    fn data<'l>(slf: Bound<'l, Self>, py: Python<'l>) -> PyResult<Bound<'l, PyAny>> {
        if slf.as_super().getattr("many")?.extract::<bool>()? {
            let mut many_result = Vec::new();
            if let Some(instances) = slf
                .getattr("instance")?
                .extract::<Option<Vec<Bound<PyAny>>>>()?
            {
                for inst in instances {
                    many_result.push(Self::to_representation(slf.clone(), inst, py)?);
                }
            }
            return Ok(PyList::new(py, many_result)?.into_any());
        } else if let Some(instance) = slf.getattr("instance")?.extract::<Option<Bound<PyAny>>>()? {
            return Ok(Self::to_representation(slf, instance, py)?.into_any());
        }
        py.None().into_bound_py_any(py)
    }
}

impl Serializer {
    fn json_schema_value(cls: &Bound<'_, PyType>) -> PyResult<Value> {
        let mut properties = serde_json::Map::new();
        let mut required_fields = Vec::new();
        let mut is_many = false;
        let mut title = None;
        let mut description = None;

        if let Ok(cls_dict) = cls.getattr("__dict__") {
            if let Ok(many) = cls_dict.get_item("many") {
                is_many = many.extract::<bool>().unwrap_or(false);
            }
            if let Ok(t) = cls_dict.get_item("title") {
                title = t.extract::<Option<String>>()?;
            }
            if let Ok(d) = cls_dict.get_item("description") {
                description = d.extract::<Option<String>>()?;
            }
        }

        for attr in cls.dir()? {
            let attr_name = attr.to_string();
            if attr_name.starts_with("_") {
                continue;
            }

            if let Ok(attr_obj) = cls.getattr(&attr_name) {
                if let Ok(field) = attr_obj.extract::<PyRef<Field>>() {
                    properties.insert(attr_name.clone(), field.to_json_schema_value());
                    if field.required.unwrap_or(false) {
                        required_fields.push(attr_name);
                    }
                    if field.many.unwrap_or(false) {
                        is_many = true;
                    }
                } else if attr_obj.extract::<PyRef<Serializer>>().is_ok() {
                    let nested_schema = Self::json_schema_value(&attr_obj.get_type())?;
                    properties.insert(attr_name.clone(), nested_schema);

                    if let Ok(field) = attr_obj.extract::<PyRef<Field>>() {
                        if field.required.unwrap_or(false) {
                            required_fields.push(attr_name);
                        }
                        if field.many.unwrap_or(false) {
                            is_many = true;
                        }
                    }
                }
            }
        }

        let mut schema = serde_json::Map::new();
        schema.insert("type".to_string(), Value::String("object".to_string()));
        schema.insert("properties".to_string(), Value::Object(properties));
        schema.insert("additionalProperties".to_string(), Value::Bool(false));

        if !required_fields.is_empty() {
            let reqs: Vec<Value> = required_fields.into_iter().map(Value::String).collect();
            schema.insert("required".to_string(), Value::Array(reqs));
        }

        if let Some(t) = title {
            schema.insert("title".to_string(), Value::String(t));
        }

        if let Some(d) = description {
            schema.insert("description".to_string(), Value::String(d));
        }

        let final_schema = if is_many {
            let mut array_schema = serde_json::Map::new();
            array_schema.insert("type".to_string(), Value::String("array".to_string()));
            array_schema.insert("items".to_string(), Value::Object(schema));
            Value::Object(array_schema)
        } else {
            Value::Object(schema)
        };

        Ok(final_schema)
    }
}

#[pymodule]
pub fn serializer_submodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let serializer = PyModule::new(m.py(), "serializer")?;
    serializer.add_class::<Field>()?;
    serializer.add_class::<EmailField>()?;
    serializer.add_class::<IntegerField>()?;
    serializer.add_class::<CharField>()?;
    serializer.add_class::<BooleanField>()?;
    serializer.add_class::<NumberField>()?;
    serializer.add_class::<UUIDField>()?;
    serializer.add_class::<DateField>()?;
    serializer.add_class::<DateTimeField>()?;
    serializer.add_class::<EnumField>()?;
    serializer.add_class::<Serializer>()?;
    m.add_submodule(&serializer)?;
    Ok(())
}
