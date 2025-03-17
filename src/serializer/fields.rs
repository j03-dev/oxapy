use pyo3::prelude::*;
use serde_json::Value;

#[pyclass(subclass)]
#[derive(Debug, Clone)]
pub struct Field {
    #[pyo3(get)]
    pub required: Option<bool>,
    #[pyo3(get)]
    pub ty: String,
    #[pyo3(get)]
    pub format: Option<String>,
    #[pyo3(get)]
    pub many: Option<bool>,
}

#[pymethods]
impl Field {
    #[new]
    #[pyo3(signature = (ty, required = true, format = None, many = false))]
    pub fn new(
        ty: String,
        required: Option<bool>,
        format: Option<String>,
        many: Option<bool>,
    ) -> Self {
        Self {
            required,
            ty,
            format,
            many,
        }
    }
}

impl Field {
    pub fn to_json_schema_value(&self) -> Value {
        let mut schema = serde_json::Map::new();
        schema.insert("type".to_string(), Value::String(self.ty.clone()));
        if let Some(fmt) = &self.format {
            schema.insert("format".to_string(), Value::String(fmt.clone()));
        }
        if self.many.unwrap_or(false) {
            let mut array_schema = serde_json::Map::new();
            array_schema.insert("type".to_string(), Value::String("array".to_string()));
            array_schema.insert("items".to_string(), Value::Object(schema));
            return Value::Object(array_schema);
        }
        Value::Object(schema)
    }
}

#[pyclass(subclass, extends=Field)]
pub struct EmailField;

#[pymethods]
impl EmailField {
    #[new]
    #[pyo3(signature=(required=true, many=false))]
    fn new(required: Option<bool>, many: Option<bool>) -> (Self, Field) {
        (
            Self,
            Field::new(
                "string".to_string(),
                required,
                Some("email".to_string()),
                many,
            ),
        )
    }
}

macro_rules! fileds {
    ($(($f:ident, $fmt:expr);)+) => {
        $(
            #[pyclass(subclass, extends=Field)]
            pub struct $f;

            #[pymethods]
            impl $f {
                #[new]
                #[pyo3(signature=(required=true, format=None, many=false))]
                fn new(required: Option<bool>, format: Option<String>, many: Option<bool>) -> (Self, Field) {
                    (
                        Self,
                        Field::new(
                            $fmt.to_string(),
                            required,
                            format,
                            many,
                        ),
                    )
                }
            }
        )+
    };
}

fileds! {
    (IntegerField, "integer");
    (CharField, "string");
}
