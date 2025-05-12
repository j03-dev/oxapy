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
    pub format: Option<String>,
    #[pyo3(get)]
    pub many: Option<bool>,
    #[pyo3(get)]
    pub min_length: Option<usize>,
    #[pyo3(get)]
    pub max_length: Option<usize>,
    #[pyo3(get)]
    pub minimum: Option<f64>,
    #[pyo3(get)]
    pub maximum: Option<f64>,
    #[pyo3(get)]
    pub pattern: Option<String>,
    #[pyo3(get)]
    pub enum_values: Option<Vec<String>>,
    #[pyo3(get)]
    pub title: Option<String>,
    #[pyo3(get)]
    pub description: Option<String>,
}

#[pymethods]
impl Field {
    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(signature = (
        ty,
        required = true,
        format = None,
        many = false,
        min_length = None,
        max_length = None,
        minimum = None,
        maximum = None,
        pattern = None,
        enum_values = None,
        title = None,
        description = None
    ))]
    pub fn new(
        ty: String,
        required: Option<bool>,
        format: Option<String>,
        many: Option<bool>,
        min_length: Option<usize>,
        max_length: Option<usize>,
        minimum: Option<f64>,
        maximum: Option<f64>,
        pattern: Option<String>,
        enum_values: Option<Vec<String>>,
        title: Option<String>,
        description: Option<String>,
    ) -> Self {
        Self {
            required,
            ty,
            format,
            many,
            min_length,
            max_length,
            minimum,
            maximum,
            pattern,
            enum_values,
            title,
            description,
        }
    }
}

static TYPE_STR: &str = "type";
static FORMAT_STR: &str = "format";
static MIN_LEN_STR: &str = "minLength";
static MAX_LEN_STR: &str = "maxLength";
static MIN_STR: &str = "minimum";
static MAX_STR: &str = "maximum";
static PATTERN_STR: &str = "pattern";
static ENUM_STR: &str = "enum";
static TITLE_STR: &str = "title";
static DESC_STR: &str = "description";
static ARRAY_STR: &str = "array";
static ITEMS_STR: &str = "items";

impl Field {
    pub fn to_json_schema_value(&self) -> Value {
        let capacity = 1
            + self.format.is_some() as usize
            + self.min_length.is_some() as usize
            + self.max_length.is_some() as usize
            + self.minimum.is_some() as usize
            + self.maximum.is_some() as usize
            + self.pattern.is_some() as usize
            + self.enum_values.is_some() as usize
            + self.title.is_some() as usize
            + self.description.is_some() as usize;

        let mut schema = serde_json::Map::with_capacity(capacity);
        schema.insert(TYPE_STR.to_string(), Value::String(self.ty.clone()));

        if let Some(fmt) = &self.format {
            schema.insert(FORMAT_STR.to_string(), Value::String(fmt.clone()));
        }

        if let Some(min_length) = self.min_length {
            schema.insert(MIN_LEN_STR.to_string(), Value::Number(min_length.into()));
        }

        if let Some(max_length) = self.max_length {
            schema.insert(MAX_LEN_STR.to_string(), Value::Number(max_length.into()));
        }

        if let Some(minimum) = self.minimum {
            schema.insert(MIN_STR.to_string(), serde_json::json!(minimum));
        }

        if let Some(maximum) = self.maximum {
            schema.insert(MAX_STR.to_string(), serde_json::json!(maximum));
        }

        if let Some(pattern) = &self.pattern {
            schema.insert(PATTERN_STR.to_string(), Value::String(pattern.clone()));
        }

        if let Some(enum_values) = &self.enum_values {
            let enum_array: Vec<Value> = enum_values
                .iter()
                .map(|v| Value::String(v.clone()))
                .collect();
            schema.insert(ENUM_STR.to_string(), Value::Array(enum_array));
        }

        if let Some(title) = &self.title {
            schema.insert(TITLE_STR.to_string(), Value::String(title.clone()));
        }

        if let Some(description) = &self.description {
            schema.insert(DESC_STR.to_string(), Value::String(description.clone()));
        }

        if self.many.unwrap_or(false) {
            let mut array_schema = serde_json::Map::with_capacity(2);
            array_schema.insert(TYPE_STR.to_string(), Value::String(ARRAY_STR.to_string()));
            array_schema.insert(ITEMS_STR.to_string(), Value::Object(schema));
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
                    format=$default_format,
                    many=false,
                    min_length=None,
                    max_length=None,
                    minimum=None,
                    maximum=None,
                    pattern=None,
                    enum_values=None,
                    title=None,
                    description=None
                ))]
                fn new(
                    required: Option<bool>,
                    format: Option<String>,
                    many: Option<bool>,
                    min_length: Option<usize>,
                    max_length: Option<usize>,
                    minimum: Option<f64>,
                    maximum: Option<f64>,
                    pattern: Option<String>,
                    enum_values: Option<Vec<String>>,
                    title: Option<String>,
                    description: Option<String>,
                ) -> (Self, Field) {
                    (
                        Self,
                        Field::new(
                            $type.to_string(),
                            required,
                            format,
                            many,
                            min_length,
                            max_length,
                            minimum,
                            maximum,
                            pattern,
                            enum_values,
                            title,
                            description,
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
