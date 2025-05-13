use pyo3::{
    create_exception,
    exceptions::{PyException, PyValueError},
    prelude::*,
    types::{PyDict, PyList, PyType},
    IntoPyObjectExt,
};
use serde_json::Value;

use once_cell::sync::Lazy;

use std::{collections::HashMap, sync::Mutex};

use crate::{request::Request, IntoPyException};

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
            Field {
                required,
                ty: "object".to_string(),
                many,
                title,
                description,
                ..Default::default()
            },
        )
    }

    fn schema(slf: Bound<'_, Self>) -> PyResult<Py<PyDict>> {
        let schema_value = Self::json_schema_value(&slf.get_type())?;
        crate::json::loads(&schema_value.to_string())
    }

    fn is_valid(slf: &Bound<'_, Self>) -> PyResult<()> {
        let request: Request = slf.getattr("request")?.extract()?;

        let json_string: String = request
            .body
            .clone()
            .ok_or_else(|| PyValueError::new_err("Request body is empty"))?;

        let attr = crate::json::loads(&json_string)?;

        let validated_data: Option<Bound<PyDict>> =
            slf.call_method1("validate", (attr,))?.extract()?;

        slf.setattr("validate_data", validated_data)?;
        Ok(())
    }

    fn validate<'a>(slf: Bound<'a, Self>, attr: Bound<'a, PyDict>) -> PyResult<Bound<'a, PyDict>> {
        let data = crate::json::dumps(&attr.clone().into())?;
        let json_value: Value = serde_json::from_str(&data).into_py_exception()?;

        let schema_value = Self::json_schema_value(&slf.get_type())?;

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

    fn create(
        slf: &Bound<Self>,
        session: PyObject,
        validate_data: Bound<PyDict>,
        py: Python<'_>,
    ) -> PyResult<()> {
        if let Ok(class_meta) = slf.getattr("Meta") {
            let model = class_meta.getattr("model")?;
            let instance = model.call((), Some(&validate_data))?;
            session.call_method1(py, "add", (instance,))?;
            session.call_method0(py, "commit")?;
        }
        Ok(())
    }

    fn save(slf: Bound<'_, Self>, session: PyObject, py: Python<'_>) -> PyResult<()> {
        let validate_data: Bound<PyDict> = slf
            .getattr("validate_data")?
            .extract::<Option<Bound<PyDict>>>()?
            .ok_or_else(|| PyException::new_err("call `is_valid()` before `save()`"))?;

        Self::create(&slf, session, validate_data, py)?;
        Ok(())
    }

    fn update(
        slf: Bound<'_, Self>,
        instance: PyObject,
        session: PyObject,
        py: Python<'_>,
    ) -> PyResult<()> {
        let validate_data = slf
            .getattr("validate_data")?
            .extract::<Option<HashMap<String, PyObject>>>()?
            .ok_or_else(|| PyException::new_err("call `is_valid()` before `save()`"))?;
        for (key, value) in validate_data {
            instance.setattr(py, key, value)?;
        }
        session.call_method0(py, "commit")?;
        Ok(())
    }
}

static CACHES_JSON_SCHEMA_VALUE: Lazy<Mutex<HashMap<String, Value>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static TYPE_STR: &str = "type";
static ARRAY_STR: &str = "array";
static OBJECT_STR: &str = "object";
static ITEMS_STR: &str = "items";
static TITLE_STR: &str = "title";
static DESC_STR: &str = "description";
static PROPS_STR: &str = "properties";
static ADD_PROPS_STR: &str = "additionalProperties";
static REQUIRED_STR: &str = "required";

impl Serializer {
    fn json_schema_value(cls: &Bound<'_, PyType>) -> PyResult<Value> {
        let mut properties = serde_json::Map::with_capacity(16);
        let mut required_fields = Vec::with_capacity(8);
        let mut is_many = false;
        let mut title = None;
        let mut description = None;

        let class_name = cls.name()?;

        if let Some(value) = CACHES_JSON_SCHEMA_VALUE
            .lock()
            .into_py_exception()?
            .get(&class_name.to_string())
            .cloned()
        {
            return Ok(value);
        }

        if let Ok(cls_dict) = cls.getattr("__dict__") {
            if let Ok(many) = cls_dict.get_item("many") {
                if let Ok(is_many_extract) = many.extract::<bool>() {
                    is_many = is_many_extract;
                }
            }
            if let Ok(t) = cls_dict.get_item("title") {
                if let Ok(titre_extract) = t.extract::<Option<String>>() {
                    title = titre_extract;
                }
            }
            if let Ok(d) = cls_dict.get_item("description") {
                if let Ok(description_extract) = d.extract::<Option<String>>() {
                    description = description_extract;
                }
            }
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

                    let nested_schema = Self::json_schema_value(&attr_obj.get_type())?;

                    if is_field_many {
                        let mut array_schema = serde_json::Map::with_capacity(2);
                        array_schema
                            .insert(TYPE_STR.to_string(), Value::String(ARRAY_STR.to_string()));
                        array_schema.insert(ITEMS_STR.to_string(), nested_schema);
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
        schema.insert(TYPE_STR.to_string(), Value::String(OBJECT_STR.to_string()));
        schema.insert(PROPS_STR.to_string(), Value::Object(properties));
        schema.insert(ADD_PROPS_STR.to_string(), Value::Bool(false));

        if !required_fields.is_empty() {
            let reqs: Vec<Value> = required_fields.into_iter().map(Value::String).collect();
            schema.insert(REQUIRED_STR.to_string(), Value::Array(reqs));
        }

        if let Some(t) = title {
            schema.insert(TITLE_STR.to_string(), Value::String(t));
        }
        if let Some(d) = description {
            schema.insert(DESC_STR.to_string(), Value::String(d));
        }

        let final_schema = if is_many {
            let mut array_schema = serde_json::Map::with_capacity(2);
            array_schema.insert(TYPE_STR.to_string(), Value::String(ARRAY_STR.to_string()));
            array_schema.insert(ITEMS_STR.to_string(), Value::Object(schema));
            Value::Object(array_schema)
        } else {
            Value::Object(schema)
        };

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
