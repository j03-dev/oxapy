use pyo3::prelude::*;
use serde_json::Value;

#[pyclass(subclass)]
#[derive(Debug, Clone, Default)]
pub struct Field {
    #[pyo3(get)]
    pub required: Option<bool>,
    #[pyo3(get)]
    pub ty: String,
    #[pyo3(get)]
    pub nullable: Option<bool>,
    #[pyo3(get)]
    pub format: Option<String>,
    #[pyo3(get)]
    pub many: Option<bool>,
    #[pyo3(get)]
    pub min_length: Option<usize>,
    #[pyo3(get)]
    pub max_length: Option<usize>,
    #[pyo3(get)]
    pub pattern: Option<String>,
    #[pyo3(get)]
    pub enum_values: Option<Vec<String>>,
}

#[pymethods]
impl Field {
    /// Creates a new `Field` instance with the specified type and optional schema constraints.
    ///
    /// Initializes a field definition for use in Python, supporting options such as required/nullable status, formatting, array handling, length limits, pattern matching, and enumerated values.
    ///
    /// # Examples
    ///
    /// ```
    /// let field = Field::new(
    ///     "string".to_string(),
    ///     Some(true),
    ///     Some(false),
    ///     Some("email".to_string()),
    ///     Some(false),
    ///     Some(3),
    ///     Some(255),
    ///     Some(r"^[a-z]+$".to_string()),
    ///     Some(vec!["foo".to_string(), "bar".to_string()])
    /// );
    /// ```
    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(signature = (
        ty,
        required = true,
        nullable = false,
        format = None,
        many = false,
        min_length = None,
        max_length = None,
        pattern = None,
        enum_values = None,
    ))]
    pub fn new(
        ty: String,
        required: Option<bool>,
        nullable: Option<bool>,
        format: Option<String>,
        many: Option<bool>,
        min_length: Option<usize>,
        max_length: Option<usize>,
        pattern: Option<String>,
        enum_values: Option<Vec<String>>,
    ) -> Self {
        Self {
            required,
            ty,
            nullable,
            format,
            many,
            min_length,
            max_length,
            pattern,
            enum_values,
        }
    }
}

impl Field {
    /// Converts the field definition into a JSON schema representation.
    ///
    /// Generates a `serde_json::Value` object describing the field as a JSON schema, including type, format, length constraints, pattern, enum values, and array/nullable handling as specified by the field's properties.
    ///
    /// # Returns
    /// A `serde_json::Value` representing the JSON schema for this field.
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

        if let Some(min_length) = self.min_length {
            schema.insert("minLength".to_string(), Value::Number(min_length.into()));
        }

        if let Some(max_length) = self.max_length {
            schema.insert("maxLength".to_string(), Value::Number(max_length.into()));
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
    ($(($class:ident, $type:expr, $default_format:expr);)+) => {
        $(
            #[pyclass(subclass, extends=Field)]
            pub struct $class;

            #[allow(clippy::too_many_arguments)]
            #[pymethods]
            impl $class {
                #[new]
                #[pyo3(signature=(
                    required=true,
                    nullable=false,
                    format=$default_format,
                    many=false,
                    min_length=None,
                    max_length=None,
                    pattern=None,
                    enum_values=None,
                ))]
                fn new(
                    required: Option<bool>,
                    nullable: Option<bool>,
                    format: Option<String>,
                    many: Option<bool>,
                    min_length: Option<usize>,
                    max_length: Option<usize>,
                    pattern: Option<String>,
                    enum_values: Option<Vec<String>>,
                ) -> (Self, Field) {
                    (
                        Self,
                        Field::new(
                            $type.to_string(),
                            required,
                            nullable,
                            format,
                            many,
                            min_length,
                            max_length,
                            pattern,
                            enum_values,
                        ),
                    )
                }
            }
        )+
    };
}

define_fields! {
    (IntegerField, "integer", None);
    (CharField, "string", None);
    (BooleanField, "boolean", None);
    (NumberField, "number", None);
    (EmailField, "string", Some("email".to_string()));
    (UUIDField, "string", Some("uuid".to_string()));
    (DateField, "string", Some("date".to_string()));
    (DateTimeField, "string", Some("date-time".to_string()));
    (EnumField, "string", None);
}
