use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;
use serde_json::Value;

/// Base class representing a JSON schema field.
#[gen_stub_pyclass]
#[pyclass(subclass, module = "oxapy.serializer")]
#[derive(Debug, Clone, Default)]
pub struct Field {
    #[pyo3(get, set)]
    pub required: Option<bool>,
    #[pyo3(get, set)]
    pub ty: String,
    #[pyo3(get, set)]
    pub nullable: Option<bool>,
    #[pyo3(get, set)]
    pub format: Option<String>,
    #[pyo3(get, set)]
    pub many: Option<bool>,
    #[pyo3(get, set)]
    pub length: Option<usize>,
    #[pyo3(get, set)]
    pub min_length: Option<usize>,
    #[pyo3(get, set)]
    pub max_length: Option<usize>,
    #[pyo3(get, set)]
    pub pattern: Option<String>,
    #[pyo3(get, set)]
    pub enum_values: Option<Vec<String>>,
    #[pyo3(get, set)]
    pub read_only: Option<bool>,
    #[pyo3(get, set)]
    pub write_only: Option<bool>,
}

#[gen_stub_pymethods]
#[pymethods]
impl Field {
    /// Create a new field definition.
    ///
    /// This is the base field class. You usually use one of the subclasses
    /// (`CharField`, `EmailField`, `IntegerField`, etc.) rather than instantiating
    /// this directly.
    ///
    /// Args:
    ///     ty (str): Field type, e.g., `"string"`, `"integer"`, etc.
    ///     required (bool, optional): Whether this field is required. Defaults to `True`.
    ///     nullable (bool, optional): Whether this field allows `null`. Defaults to `False`.
    ///     format (str, optional): Optional format (e.g., `"email"`, `"uuid"`).
    ///     many (bool, optional): Whether this field is a list of values. Defaults to `False`.
    ///     length (int, optional): Length for string fields.
    ///     min_length (int, optional): Minimum length for string fields.
    ///     max_length (int, optional): Maximum length for string fields.
    ///     pattern (str, optional): Regular expression pattern for validation.
    ///     enum_values (list[str], optional): List of allowed values.
    ///     read_only (bool, optional): If `True`, the field will be excluded when deserializing. Defaults to `None`.
    ///     write_only (bool, optional): If `True`, the field will be excluded when serializing. Defaults to `None`.
    ///
    /// Example:
    /// ```python
    /// field = Field("string", min_length=3, max_length=255)
    /// ```
    #[pyo3(signature = (
        ty,
        required = Some(true),
        nullable = Some(false),
        format = None,
        many = Some(false),
        length = None,
        min_length = None,
        max_length = None,
        pattern = None,
        enum_values = None,
        read_only = None,
        write_only = None
    ))]
    #[allow(clippy::too_many_arguments)]
    #[new]
    pub fn new(
        ty: String,
        required: Option<bool>,
        nullable: Option<bool>,
        format: Option<String>,
        many: Option<bool>,
        length: Option<usize>,
        min_length: Option<usize>,
        max_length: Option<usize>,
        pattern: Option<String>,
        enum_values: Option<Vec<String>>,
        read_only: Option<bool>,
        write_only: Option<bool>,
    ) -> Self {
        Self {
            required,
            ty,
            nullable,
            format,
            many,
            length,
            min_length,
            max_length,
            pattern,
            enum_values,
            read_only,
            write_only,
        }
    }
}

impl Field {
    pub fn to_json_schema_value(&self) -> Value {
        let capacity = 1
            + self.format.is_some() as usize
            + self.min_length.is_some() as usize
            + self.max_length.is_some() as usize
            + self.pattern.is_some() as usize
            + self.enum_values.is_some() as usize;

        let mut schema = serde_json::Map::with_capacity(capacity);

        if self.nullable.unwrap_or(false) {
            schema.insert("type".to_string(), serde_json::json!([self.ty, "null"]));
        } else {
            schema.insert("type".to_string(), Value::String(self.ty.clone()));
        }

        if let Some(fmt) = &self.format {
            schema.insert("format".to_string(), Value::String(fmt.clone()));
        }

        if let Some(length) = self.length {
            schema.insert("minLength".to_string(), Value::Number(length.into()));
            schema.insert("maxLength".to_string(), Value::Number(length.into()));
        } else {
            if let Some(min_length) = self.min_length {
                schema.insert("minLength".to_string(), Value::Number(min_length.into()));
            }
            if let Some(max_length) = self.max_length {
                schema.insert("maxLength".to_string(), Value::Number(max_length.into()));
            }
        }

        if let Some(pattern) = &self.pattern {
            schema.insert("pattern".to_string(), Value::String(pattern.clone()));
        }

        if let Some(enum_values) = &self.enum_values {
            let enum_array: Vec<Value> = enum_values
                .iter()
                .map(|v| Value::String(v.clone()))
                .collect();
            schema.insert("enum".to_string(), Value::Array(enum_array));
        }

        if self.many.unwrap_or(false) {
            let mut array_schema = serde_json::Map::with_capacity(2);

            if self.nullable.unwrap_or(false) {
                array_schema.insert("type".to_string(), serde_json::json!(["array", "null"]));
            } else {
                array_schema.insert("type".to_string(), Value::String("array".to_string()));
            }

            array_schema.insert("items".to_string(), Value::Object(schema));
            return Value::Object(array_schema);
        }

        Value::Object(schema)
    }
}

macro_rules! define_fields {
    ($((
        $class:ident,
        $type:expr,
        $default_format:expr,
        $doc:literal
    );)+) => {
        $(
            #[doc = $doc]
            #[gen_stub_pyclass]
            #[pyclass(module="oxapy.serializer", subclass, extends=Field)]
            pub struct $class;

            #[gen_stub_pymethods]
            #[pymethods]
            #[allow(clippy::too_many_arguments)]
            impl $class {
                /// Create a new field of this type.
                ///
                /// Args:
                ///     required (bool, optional): Whether this field is required. Defaults to `True`.
                ///     nullable (bool, optional): Whether this field allows `null`. Defaults to `False`.
                ///     format (str, optional): Optional format override.
                ///     many (bool, optional): Whether this field is a list of values.
                ///     min_length (int, optional): Minimum length (for string types).
                ///     max_length (int, optional): Maximum length (for string types).
                ///     pattern (str, optional): Regular expression pattern.
                ///     enum_values (list[str], optional): List of allowed values.
                ///     read_only (bool, optional): If `True`, the field will be excluded when deserializing.
                ///     write_only (bool, optional): If `True`, the field will be excluded when serializing.
                #[new]
                #[pyo3(signature=(
                    required=Some(true),
                    nullable=Some(false),
                    format=$default_format,
                    many=Some(false),
                    length=None,
                    min_length=None,
                    max_length=None,
                    pattern=None,
                    enum_values=None,
                    read_only=None,
                    write_only=None
                ))]
                fn new(
                    required: Option<bool>,
                    nullable: Option<bool>,
                    format: Option<String>,
                    many: Option<bool>,
                    length: Option<usize>,
                    min_length: Option<usize>,
                    max_length: Option<usize>,
                    pattern: Option<String>,
                    enum_values: Option<Vec<String>>,
                    read_only: Option<bool>,
                    write_only: Option<bool>
                ) -> ($class, Field) {
                    (
                        Self,
                        Field::new(
                            $type.to_string(),
                            required,
                            nullable,
                            format,
                            many,
                            length,
                            min_length,
                            max_length,
                            pattern,
                            enum_values,
                            read_only,
                            write_only,
                        ),
                    )
                }
            }
        )+
    };
}

define_fields! {
    (IntegerField, "integer", None, "Represents an integer field in JSON schema.");
    (CharField, "string", None, "Represents a string field.");
    (BooleanField, "boolean", None, "Represents a boolean field.");
    (NumberField, "number", None, "Represents a numeric (float) field.");
    (EmailField, "string", Some("email".to_string()), "Represents an email field, validated by format.");
    (UUIDField, "string", Some("uuid".to_string()), "Represents a UUID field.");
    (DateField, "string", Some("date".to_string()), "Represents a date field (YYYY-MM-DD).");
    (DateTimeField, "string", Some("date-time".to_string()), "Represents a date-time field (RFC 3339).");
    (EnumField, "string", None, "Represents an enumerated string field.");
}
