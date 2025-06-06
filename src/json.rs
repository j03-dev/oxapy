use pyo3::{prelude::*, types::PyDict};
use serde::{Deserialize, Serialize};

pub fn dumps(data: &PyObject) -> PyResult<String> {
    Python::with_gil(|py| {
        let orjson_module = PyModule::import(py, "orjson")?;
        let serialized_data = orjson_module
            .call_method1("dumps", (data,))?
            .call_method1("decode", ("utf-8",))?;
        serialized_data.extract()
    })
}

pub fn loads(data: &str) -> PyResult<Py<PyDict>> {
    Python::with_gil(|py| {
        let orjson_module = PyModule::import(py, "orjson")?;
        let deserialized_data = orjson_module.call_method1("loads", (data,))?;
        deserialized_data.extract()
    })
}

pub struct Wrap<T>(pub T);

impl<T> TryFrom<Bound<'_, PyDict>> for Wrap<T>
where
    T: for<'de> Deserialize<'de>,
{
    type Error = PyErr;

    fn try_from(value: Bound<'_, PyDict>) -> Result<Self, Self::Error> {
        let json_string = dumps(&value.into())?;
        let value = serde_json::from_str(&json_string)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Wrap(value))
    }
}

impl<T> TryFrom<Wrap<T>> for Py<PyDict>
where
    T: Serialize,
{
    type Error = PyErr;

    fn try_from(value: Wrap<T>) -> Result<Self, Self::Error> {
        let json_string = serde_json::json!(value.0).to_string();
        loads(&json_string)
    }
}
