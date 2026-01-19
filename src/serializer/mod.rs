mod fields;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use self::fields::*;
use crate::{IntoPyException, exceptions::ClientError, json};

use pyo3::{
    IntoPyObjectExt,
    exceptions::PyException,
    prelude::*,
    sync::PyOnceLock,
    types::{PyDict, PyList, PyType},
};
use pyo3_stub_gen::derive::*;
use serde_json::{Value, json};

static SQL_ALCHEMY_INSPECT: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

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
    context: Py<PyDict>,
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
        required = true,
        nullable = false,
        many = false,
        context = None,
        read_only= false,
        write_only = false,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        data: Option<String>,
        instance: Option<Py<PyAny>>,
        required: bool,
        nullable: bool,
        many: bool,
        context: Option<Py<PyDict>>,
        read_only: bool,
        write_only: bool,
        py: Python<'_>,
    ) -> (Serializer, Field) {
        (
            Self {
                validated_data: PyDict::new(py).into(),
                raw_data: data,
                instance,
                context: context.unwrap_or_else(|| PyDict::new(py).into()),
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
    fn schema(slf: Bound<'_, Self>, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let schema_value = Self::json_schema_value(&slf.get_type(), false, py)?;
        json::loads(&schema_value.to_string(), py)
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
    fn is_valid(slf: &Bound<'_, Self>, py: Python<'_>) -> PyResult<()> {
        let raw_data = slf
            .getattr("raw_data")?
            .extract::<Option<String>>()?
            .ok_or_else(|| ValidationException::new_err("data is empty"))?;

        let attr = json::loads(&raw_data, py)?;

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
    fn validate<'a>(
        slf: Bound<'a, Self>,
        attr: Bound<'a, PyDict>,
        py: Python<'a>,
    ) -> PyResult<Bound<'a, PyDict>> {
        let json_value = json::from_pydict2rstruct(&attr)?;

        let schema_value = Self::json_schema_value(&slf.get_type(), false, py)?;

        let validator = jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)
            .into_py_exception()?;

        validator
            .validate(&json_value)
            .map_err(|err| ValidationException::new_err(err.to_string()))?;

        for k in attr.keys() {
            let key = k.to_string();
            if let Ok(f) = slf.getattr(&key)
                && let Ok(field) = f.extract::<Field>()
                && field.read_only
            {
                attr.del_item(&key)?;
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
        if !validated_data.is_empty() {
            let instance = slf
                .call_method1("create", (session, validated_data))?
                .into();
            Ok(instance)
        } else {
            Err(PyException::new_err("call `is_valid()` before `save()`"))
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
        session.call_method1(py, "refresh", (instance.clone_ref(py),))?;
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

        let inspect = SQL_ALCHEMY_INSPECT.get_or_try_init(py, || {
            let sqlalchemy = PyModule::import(py, "sqlalchemy")?;
            let inspection = sqlalchemy.getattr("inspection")?;
            inspection.getattr("inspect").map(|i| i.into())
        })?;

        let mapper = inspect.call1(py, (instance.get_type(),))?;

        let columns = mapper.getattr(py, "columns")?.into_bound(py).try_iter()?;

        for c in columns {
            let col = c?.getattr("name")?.to_string();
            if let Ok(field) = slf.getattr(&col)
                && !field.extract::<Field>()?.write_only
            {
                dict.set_item(&col, instance.getattr(&col)?)?;
            }
        }

        let relationships = mapper
            .getattr(py, "relationships")?
            .into_bound(py)
            .try_iter()?;

        for r in relationships {
            let key = r?.getattr("key")?.to_string();
            if let Ok(field) = slf.getattr(&key)
                && !field.extract::<Field>()?.write_only
            {
                slf.getattr("context")
                    .and_then(|ctx| field.setattr("context", ctx))?;
                field.setattr("instance", instance.getattr(&key)?)?;
                dict.set_item(key, field.getattr("data")?)?;
            }
        }
        Ok(dict)
    }
}

static CACHE: PyOnceLock<Arc<Mutex<HashMap<String, Value>>>> = PyOnceLock::new();

fn cache(py: Python<'_>) -> &Arc<Mutex<HashMap<String, Value>>> {
    CACHE.get_or_init(py, || Arc::new(Mutex::new(HashMap::new())))
}

impl Serializer {
    fn json_schema_value(
        cls: &Bound<'_, PyType>,
        nullable: bool,
        py: Python<'_>,
    ) -> PyResult<Value> {
        let mut properties = serde_json::Map::with_capacity(16);
        let mut required_fields = Vec::with_capacity(8);

        let class_name = cls.name()?;

        if let Some(value) = cache(py)
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
                    field
                        .required
                        .then(|| required_fields.push(attr_name.clone()));
                    let nested_schema =
                        Self::json_schema_value(&attr_obj.get_type(), field.nullable, py)?;

                    if field.many {
                        let mut array_schema = serde_json::Map::with_capacity(2);
                        if field.nullable {
                            array_schema.insert("type".to_string(), json!(["array", "null"]))
                        } else {
                            array_schema.insert("type".to_string(), json!("array"))
                        };
                        array_schema.insert("items".to_string(), nested_schema);
                        properties.insert(attr_name, json!(array_schema));
                    } else {
                        properties.insert(attr_name, nested_schema);
                    }
                } else if let Ok(f) = attr_obj.extract::<PyRef<Field>>() {
                    properties.insert(attr_name.clone(), f.to_json_schema_value());
                    f.required.then(|| required_fields.push(attr_name));
                }
            }
        }

        let mut schema = serde_json::Map::with_capacity(5);
        if nullable {
            schema.insert("type".to_string(), json!(["object", "null"]))
        } else {
            schema.insert("type".to_string(), json!("object"))
        };
        schema.insert("properties".to_string(), json!(properties));
        schema.insert("additionalProperties".to_string(), json!(false));

        if !required_fields.is_empty() {
            schema.insert("required".to_string(), json!(required_fields));
        }

        let final_schema = json!(schema);

        cache(py)
            .lock()
            .into_py_exception()?
            .insert(class_name.to_string(), final_schema.clone());

        Ok(final_schema)
    }
}

/// Serializer validation exception.
///
/// Raised when data validation fails during serialization or deserialization.
/// This includes missing required fields, invalid field values, type mismatches,
/// and schema constraint violations.
#[gen_stub_pyclass]
#[pyclass(module = "oxapy.serializer", extends=ClientError)]
pub struct ValidationException;
extend_exception!(ValidationException, ClientError);

pub fn serializer_submodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    let serializer = PyModule::new(py, "serializer")?;
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
