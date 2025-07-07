use pyo3::{
    create_exception,
    exceptions::PyException,
    prelude::*,
    types::{PyDict, PyList, PyType},
    IntoPyObjectExt,
};
use serde_json::Value;

use once_cell::sync::Lazy;

use std::{collections::HashMap, sync::Mutex};

use crate::{json, IntoPyException};

use fields::{
    BooleanField, CharField, DateField, DateTimeField, EmailField, EnumField, Field, IntegerField,
    NumberField, UUIDField,
};

mod fields;

create_exception!(
    serializer,
    ValidationException,
    PyException,
    "Validation Exception"
);

#[pyclass(subclass, extends=Field)]
#[derive(Debug)]
struct Serializer {
    #[pyo3(get, set)]
    instance: Option<Py<PyAny>>,
    #[pyo3(get, set)]
    validated_data: Option<Py<PyDict>>,
    #[pyo3(get, set)]
    raw_data: Option<String>,
    #[pyo3(get, set)]
    context: Option<Py<PyDict>>,
}

#[pymethods]
impl Serializer {
    #[new]
    #[pyo3(signature = (
        data = None,
        instance = None,
        required = true,
        nullable = false,
        many = false,
        context = None
    ))]
    fn new(
        data: Option<String>,
        instance: Option<Py<PyAny>>,
        required: Option<bool>,
        nullable: Option<bool>,
        many: Option<bool>,
        context: Option<Py<PyDict>>,
    ) -> (Self, Field) {
        (
            Self {
                validated_data: None,
                raw_data: data,
                instance,
                context,
            },
            Field {
                required,
                ty: "object".to_string(),
                nullable,
                many,
                ..Default::default()
            },
        )
    }

    fn schema(slf: Bound<'_, Self>) -> PyResult<Py<PyDict>> {
        let schema_value = Self::json_schema_value(&slf.get_type(), None)?;
        json::loads(&schema_value.to_string())
    }

    fn is_valid(slf: &Bound<'_, Self>) -> PyResult<()> {
        let raw_data = slf
            .getattr("raw_data")?
            .extract::<Option<String>>()?
            .ok_or_else(|| ValidationException::new_err("data is empty"))?;

        let attr = json::loads(&raw_data)?;

        let validated_data: Option<Bound<PyDict>> =
            slf.call_method1("validate", (attr,))?.extract()?;

        slf.setattr("validated_data", validated_data)?;
        Ok(())
    }

    fn validate<'a>(slf: Bound<'a, Self>, attr: Bound<'a, PyDict>) -> PyResult<Bound<'a, PyDict>> {
        let json::Wrap(json_value) = attr.clone().try_into()?;

        let schema_value = Self::json_schema_value(&slf.get_type(), None)?;

        let validator = jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)
            .into_py_exception()?;

        validator
            .validate(&json_value)
            .map_err(|err| ValidationException::new_err(err.to_string()))?;

        Ok(attr)
    }

    fn to_representation<'l>(
        slf: &Bound<'_, Self>,
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
    fn data<'l>(slf: Bound<'l, Self>, py: Python<'l>) -> PyResult<PyObject> {
        let many = slf.getattr("many")?.extract::<bool>()?;
        if many {
            let mut results: Vec<PyObject> = Vec::new();
            if let Some(instances) = slf
                .getattr("instance")?
                .extract::<Option<Vec<PyObject>>>()?
            {
                for instance in instances {
                    let repr = slf.call_method1("to_representation", (instance,))?;
                    results.push(repr.extract()?);
                }
            }
            return PyList::new(py, results)?.into_py_any(py);
        }

        if let Some(instance) = slf.getattr("instance")?.extract::<Option<PyObject>>()? {
            let repr = slf.call_method1("to_representation", (instance,))?;
            return repr.extract();
        }

        Ok(py.None())
    }

    fn create<'l>(
        slf: &'l Bound<Self>,
        session: PyObject,
        validated_data: Bound<PyDict>,
        py: Python<'l>,
    ) -> PyResult<PyObject> {
        let class_meta = slf.getattr("Meta")?;
        let model = class_meta.getattr("model")?;
        let instance = model.call((), Some(&validated_data))?;
        session.call_method1(py, "add", (instance.clone(),))?;
        session.call_method0(py, "commit")?;
        Ok(instance.into())
    }

    fn save(slf: Bound<'_, Self>, session: PyObject) -> PyResult<PyObject> {
        let validated_data: Bound<PyDict> = slf
            .getattr("validated_data")?
            .extract::<Option<Bound<PyDict>>>()?
            .ok_or_else(|| PyException::new_err("call `is_valid()` before `save()`"))?;
        Ok(slf
            .call_method1("create", (session, validated_data))?
            .into())
    }

    fn update(
        &self,
        session: PyObject,
        instance: PyObject,
        validated_data: HashMap<String, PyObject>,
        py: Python<'_>,
    ) -> PyResult<PyObject> {
        for (key, value) in validated_data {
            instance.setattr(py, key, value)?;
        }
        session.call_method0(py, "commit")?;
        Ok(instance)
    }
}

static CACHES_JSON_SCHEMA_VALUE: Lazy<Mutex<HashMap<String, Value>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

impl Serializer {
    fn json_schema_value(cls: &Bound<'_, PyType>, nullable: Option<bool>) -> PyResult<Value> {
        let mut properties = serde_json::Map::with_capacity(16);
        let mut required_fields = Vec::with_capacity(8);

        let class_name = cls.name()?;

        if let Some(value) = CACHES_JSON_SCHEMA_VALUE
            .lock()
            .into_py_exception()?
            .get(&class_name.to_string())
            .cloned()
        {
            return Ok(value);
        }

        let attrs = cls.dir()?;
        for attr in attrs.iter() {
            let attr_name = attr.to_string();
            if attr_name.starts_with('_') {
                continue;
            }

            if let Ok(attr_obj) = cls.getattr(&attr_name) {
                if let Ok(serializer) = attr_obj.extract::<PyRef<Serializer>>() {
                    let field = serializer.as_super();
                    let is_required = field.required.unwrap_or(false);
                    let is_field_many = field.many.unwrap_or(false);

                    if is_required {
                        required_fields.push(attr_name.clone());
                    }

                    let nested_schema =
                        Self::json_schema_value(&attr_obj.get_type(), field.nullable)?;

                    if is_field_many {
                        let mut array_schema = serde_json::Map::with_capacity(2);

                        if field.nullable.unwrap_or(false) {
                            array_schema
                                .insert("type".to_string(), serde_json::json!(["array", "null"]));
                        } else {
                            array_schema
                                .insert("type".to_string(), Value::String("array".to_string()));
                        }

                        array_schema.insert("items".to_string(), nested_schema);
                        properties.insert(attr_name, Value::Object(array_schema));
                    } else {
                        properties.insert(attr_name, nested_schema);
                    }
                } else if let Ok(field) = attr_obj.extract::<PyRef<Field>>() {
                    properties.insert(attr_name.clone(), field.to_json_schema_value());

                    if field.required.unwrap_or(false) {
                        required_fields.push(attr_name);
                    }
                }
            }
        }

        let mut schema = serde_json::Map::with_capacity(5);
        if nullable.unwrap_or_default() {
            schema.insert("type".to_string(), serde_json::json!(["object", "null"]));
        } else {
            schema.insert("type".to_string(), Value::String("object".to_string()));
        }
        schema.insert("properties".to_string(), Value::Object(properties));
        schema.insert("additionalProperties".to_string(), Value::Bool(false));

        if !required_fields.is_empty() {
            let reqs: Vec<Value> = required_fields.into_iter().map(Value::String).collect();
            schema.insert("required".to_string(), Value::Array(reqs));
        }

        let final_schema = Value::Object(schema);

        CACHES_JSON_SCHEMA_VALUE
            .lock()
            .into_py_exception()?
            .insert(class_name.to_string(), final_schema.clone());

        Ok(final_schema)
    }
}

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
    serializer.add(
        "ValidationException",
        m.py().get_type::<ValidationException>(),
    )?;
    m.add_submodule(&serializer)?;
    Ok(())
}
