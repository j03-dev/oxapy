use pyo3::{
    exceptions::PyException,
    impl_exception_boilerplate,
    prelude::*,
    types::{PyDict, PyList, PyType},
    IntoPyObjectExt,
};
use pyo3_stub_gen::derive::*;
use serde_json::Value;

use once_cell::sync::{Lazy, OnceCell};

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{exceptions::ClientError, json, IntoPyException};

use fields::{
    BooleanField, CharField, DateField, DateTimeField, EmailField, EnumField, Field, IntegerField,
    NumberField, UUIDField,
};

mod fields;

#[gen_stub_pyclass]
#[pyclass(module="oxapy.serializer", subclass, extends=Field)]
#[derive(Debug)]
struct Serializer {
    #[pyo3(get, set)]
    instance: Option<Py<PyAny>>,
    #[pyo3(get, set)]
    validated_data: Py<PyDict>,
    #[pyo3(get, set)]
    raw_data: Option<String>,
    #[pyo3(get, set)]
    context: Option<Py<PyDict>>,
}

#[gen_stub_pymethods]
#[pymethods]
impl Serializer {
    /// Create a new `Serializer` instance.
    ///
    /// This constructor initializes the serializer with optional raw JSON data, an instance to serialize,
    /// and optional context.
    ///
    /// Args:
    ///     data (str, optional): Raw JSON string to be validated or deserialized.
    ///     instance (Any, optional): Python object instance to be serialized.
    ///     required (bool, optional): Whether the field is required (default: True).
    ///     nullable (bool, optional): Whether the field allows null values (default: False).
    ///     many (bool, optional): Whether the serializer handles multiple objects (default: False).
    ///     context (dict, optional): Additional context information.
    ///     read_only (bool, optional): If `True`, the serializer will be excluded when deserializing (default: False).
    ///     write_only (bool, optional): If `True`, the serializer will be excluded when serializing (default: False).
    ///
    /// Returns:
    ///     Serializer: The new serializer instance.
    ///
    /// Example:
    /// ```python
    /// from oxapy import serializer
    ///
    /// class MySerializer(serializer.Serializer):
    ///     email = serializer.EmailField()
    ///     password = serializer.CharField(write_only=True)
    ///
    /// serializer = MySerializer(
    ///     data='{"email": "user@example.com", "password": "secret123"}'
    /// )
    /// ```
    #[new]
    #[pyo3(signature = (
        data = None,
        instance = None,
        required = Some(true),
        nullable = Some(false),
        many = Some(false),
        context = None,
        read_only= Some(false),
        write_only = Some(false),
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        data: Option<String>,
        instance: Option<Py<PyAny>>,
        required: Option<bool>,
        nullable: Option<bool>,
        many: Option<bool>,
        context: Option<Py<PyDict>>,
        read_only: Option<bool>,
        write_only: Option<bool>,
        py: Python<'_>,
    ) -> (Serializer, Field) {
        (
            Self {
                validated_data: PyDict::new(py).into(),
                raw_data: data,
                instance,
                context,
            },
            Field {
                required,
                ty: "object".to_string(),
                nullable,
                many,
                read_only,
                write_only,
                ..Default::default()
            },
        )
    }

    /// Generate and return the JSON Schema for this serializer.
    ///
    /// The schema is built dynamically based on the serializer class definition and its fields.
    ///
    /// Returns:
    ///     dict: The JSON Schema as a Python dictionary.
    ///
    /// Example:
    /// ```python
    /// from oxapy import serializer
    ///
    /// class MySerializer(serializer.Serializer):
    ///     email = serializer.EmailField()
    ///
    /// serializer = MySerializer()
    /// schema = serializer.schema()
    /// print(schema)
    /// ```
    #[pyo3(signature=())]
    fn schema(slf: Bound<'_, Self>) -> PyResult<Py<PyDict>> {
        let schema_value = Self::json_schema_value(&slf.get_type(), None)?;
        json::loads(&schema_value.to_string())
    }

    /// Validate the raw JSON data and store the result in `validated_data`.
    ///
    /// Parses the `raw_data` JSON string, validates it, and saves the result as `validated_data`.
    ///
    /// Raises:
    ///     ValidationException: If `raw_data` is missing or invalid.
    ///
    /// Example:
    /// ```python
    /// from oxapy import serializer
    ///
    /// class MySerializer(serializer.Serializer):
    ///     email = serializer.EmailField()
    ///
    /// serializer = MySerializer(data='{"email": "user@example.com"}')
    /// serializer.is_valid()
    /// print(serializer.validated_data["email"])
    /// ```
    #[pyo3(signature=())]
    fn is_valid(slf: &Bound<'_, Self>) -> PyResult<()> {
        let raw_data = slf
            .getattr("raw_data")?
            .extract::<Option<String>>()?
            .ok_or_else(|| ValidationException::new_err("data is empty"))?;

        let attr = json::loads(&raw_data)?;

        let validated_data: Bound<PyDict> = slf.call_method1("validate", (attr,))?.extract()?;

        slf.setattr("validated_data", validated_data)?;
        Ok(())
    }

    /// Validate a Python dictionary against the serializer's schema.
    ///
    /// Args:
    ///     attr (dict): The data to validate.
    ///
    /// Returns:
    ///     dict: The validated data, with any `read_only` fields removed.
    ///
    /// Raises:
    ///     ValidationException: If validation fails.
    ///
    /// Example:
    /// ```python
    /// from oxapy import serializer
    ///
    /// class MySerializer(serializer.Serializer):
    ///     email = serializer.EmailField()
    ///
    /// serializer = MySerializer()
    /// serializer.validate({"email": "user@example.com"})
    /// ```
    #[pyo3(signature=(attr))]
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

        for k in attr.keys() {
            let key = k.to_string();
            if let Ok(field) = slf.getattr(&key) {
                let field = field.extract::<Field>()?;
                if field.read_only.unwrap_or_default() {
                    attr.del_item(&key)?;
                }
            }
        }

        Ok(attr)
    }

    /// Return the serialized representation of the instance(s).
    ///
    /// If `many=True`, returns a list of serialized dicts.
    /// Otherwise, returns a single dict, or None if no instance.
    /// Fields marked as `write_only=True` will be excluded from the serialized output.
    ///
    /// Returns:
    ///     dict or list[dict] or None: Serialized representation(s).
    ///
    /// Example:
    /// ```python
    /// from oxapy import serializer
    ///
    /// class MySerializer(serializer.Serializer):
    ///     email = serializer.EmailField()
    ///
    /// class User:
    ///     def __init__(self, email):
    ///         self.email = email
    ///
    /// user = User("user@example.com")
    /// serializer = MySerializer(instance=user)
    /// print(serializer.data)
    /// ```
    #[getter]
    fn data<'l>(slf: Bound<'l, Self>, py: Python<'l>) -> PyResult<Py<PyAny>> {
        let many = slf.getattr("many")?.extract::<bool>()?;
        if many {
            let mut results: Vec<Py<PyAny>> = Vec::new();
            if let Some(instances) = slf
                .getattr("instance")?
                .extract::<Option<Vec<Py<PyAny>>>>()?
            {
                for instance in instances {
                    let repr = slf.call_method1("to_representation", (instance,))?;
                    results.push(repr.extract()?);
                }
            }
            return PyList::new(py, results)?.into_py_any(py);
        }

        if let Some(instance) = slf.getattr("instance")?.extract::<Option<Py<PyAny>>>()? {
            let repr = slf.call_method1("to_representation", (instance,))?;
            return Ok(repr.extract()?);
        }

        Ok(py.None())
    }

    /// Create and persist a new model instance with validated data.
    ///
    /// Args:
    ///     session (Any): The database session.
    ///     validated_data (dict): The validated data.
    ///
    /// Returns:
    ///     Any: The created instance.
    ///
    /// Example:
    /// ```python
    /// from oxapy import serializer
    ///
    /// class MySerializer(serializer.Serializer):
    ///     email = serializer.EmailField()
    ///
    /// serializer = MySerializer(data='{"email": "user@example.com"}')
    /// serializer.is_valid()
    /// # Assuming `session` is a database session
    /// instance = serializer.create(session, serializer.validated_data)
    /// ```
    #[pyo3(signature=(session, validated_data))]
    fn create<'l>(
        slf: &'l Bound<Self>,
        session: Py<PyAny>,
        validated_data: Bound<PyDict>,
        py: Python<'l>,
    ) -> PyResult<Py<PyAny>> {
        let class_meta = slf.getattr("Meta")?;
        let model = class_meta.getattr("model")?;
        let instance = model.call((), Some(&validated_data))?;
        session.call_method1(py, "add", (&instance,))?;
        session.call_method0(py, "commit")?;
        session.call_method1(py, "refresh", (&instance,))?;
        Ok(instance.into())
    }

    /// Save validated data by creating a new instance and persisting it.
    ///
    /// Calls `is_valid()` first to populate `validated_data` before calling `create()`.
    ///
    /// Args:
    ///     session (Any): The database session.
    ///
    /// Returns:
    ///     Any: The created instance.
    ///
    /// Raises:
    ///     Exception: If `is_valid()` was not called first.
    ///
    /// Example:
    /// ```python
    /// from oxapy import serializer
    ///
    /// class MySerializer(serializer.Serializer):
    ///     email = serializer.EmailField()
    ///
    /// serializer = MySerializer(data='{"email": "user@example.com"}')
    /// serializer.is_valid()
    /// # Assuming `session` is a database session
    /// instance = serializer.save(session)
    /// ```
    #[pyo3(signature=(session))]
    fn save(slf: Bound<'_, Self>, session: Py<PyAny>) -> PyResult<Py<PyAny>> {
        let validated_data: Bound<PyDict> = slf.getattr("validated_data")?.extract()?;
        match !validated_data.is_empty() {
            true => Ok(slf
                .call_method1("create", (session, validated_data))?
                .into()),
            false => Err(PyException::new_err("call `is_valid()` before `save()`")),
        }
    }

    /// Update an existing instance with validated data.
    ///
    /// Args:
    ///     session (Any): The database session.
    ///     instance (Any): The instance to update.
    ///     validated_data (dict): Field names and new values.
    ///
    /// Returns:
    ///     Any: The updated instance.
    ///
    /// Example:
    /// ```python
    /// from oxapy import serializer
    ///
    /// class MySerializer(serializer.Serializer):
    ///     email = serializer.EmailField()
    ///
    /// # Assuming `session` and `instance` are available
    /// serializer = MySerializer()
    /// updated = serializer.update(session, instance, {"email": "new@email.com"})
    /// ```
    fn update(
        &self,
        session: Py<PyAny>,
        instance: Py<PyAny>,
        validated_data: HashMap<String, Py<PyAny>>,
        py: Python<'_>,
    ) -> PyResult<Py<PyAny>> {
        for (key, value) in validated_data {
            instance.setattr(py, key, value)?;
        }
        session.call_method0(py, "commit")?;
        Ok(instance)
    }

    /// Convert a model instance to a Python dictionary.
    ///
    /// Processes each field in the model, excluding those marked as `write_only=True`.
    ///
    /// Args:
    ///     instance: The model instance to serialize.
    ///
    /// Returns:
    ///     dict: Dictionary representation of the instance.
    #[pyo3(signature=(instance))]
    #[inline]
    fn to_representation<'l>(
        slf: Bound<'_, Self>,
        instance: Bound<PyAny>,
        py: Python<'l>,
    ) -> PyResult<Bound<'l, PyDict>> {
        let dict = PyDict::new(py);

        let inspect = INSPECT
            .get()
            .ok_or_else(|| PyException::new_err("sqlalchemy is not installed"))?;

        let mapper = inspect.call1(py, (instance.get_type(),))?;

        let columns = mapper.getattr(py, "columns")?.into_bound(py).try_iter()?;
        let relationships = mapper
            .getattr(py, "relationships")?
            .into_bound(py)
            .try_iter()?;

        for c in columns {
            let col = c?.getattr("name")?.to_string();
            if let Ok(field) = slf.getattr(&col) {
                if !field.extract::<Field>()?.write_only.unwrap_or_default() {
                    dict.set_item(&col, instance.getattr(&col)?)?;
                }
            }
        }

        for r in relationships {
            let key = r?.getattr("key")?.to_string();
            if let Ok(field) = slf.getattr(&key) {
                if !field.extract::<Field>()?.write_only.unwrap_or_default() {
                    slf.getattr("context")
                        .and_then(|ctx| field.setattr("context", ctx))?;
                    field.setattr("instance", instance.getattr(&key)?)?;
                    dict.set_item(key, field.getattr("data")?)?;
                }
            }
        }
        Ok(dict)
    }
}

static CACHES_JSON_SCHEMA_VALUE: Lazy<Arc<Mutex<HashMap<String, Value>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

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

static INSPECT: OnceCell<Py<PyAny>> = OnceCell::new();

/// Serializer validation exception.
///
/// Raised when data validation fails during serialization or deserialization.
/// This includes missing required fields, invalid field values, type mismatches,
/// and schema constraint violations.
#[gen_stub_pyclass]
#[pyclass(module = "oxapy.serializer", extends=ClientError)]
pub struct ValidationException;

impl_exception_boilerplate!(ValidationException);
extend_exception!(ValidationException, ClientError);

pub fn serializer_submodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    let serializer = PyModule::new(py, "serializer")?;

    if let Ok(sqlalchemy) = PyModule::import(py, "sqlalchemy") {
        let inspection = sqlalchemy.getattr("inspection")?;
        let inspect = inspection.getattr("inspect")?;
        INSPECT.set(inspect.into()).ok();
    }

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
