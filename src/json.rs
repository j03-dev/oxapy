use once_cell::sync::OnceCell;
use pyo3::{prelude::*, types::PyDict};
use serde::{Deserialize, Serialize};

static ORJSON: OnceCell<Py<PyModule>> = OnceCell::new();

pub fn init_orjson(py: Python<'_>) -> PyResult<()> {
    let orjson = PyModule::import(py, "orjson")?;
    ORJSON.set(orjson.into()).ok();
    Ok(())
}

#[inline]
pub fn dumps(data: &Py<PyAny>) -> PyResult<String> {
    Python::attach(|py| {
        let orjson_module = ORJSON.get().unwrap();
        let serialized_data = orjson_module
            .call_method1(py, "dumps", (data,))?
            .call_method1(py, "decode", ("utf-8",))?;
        Ok(serialized_data.extract(py)?)
    })
}

#[inline]
pub fn loads(data: &str) -> PyResult<Py<PyDict>> {
    Python::attach(|py| {
        let orjson_module = ORJSON.get().unwrap();
        let deserialized_data = orjson_module.call_method1(py, "loads", (data,))?;
        Ok(deserialized_data.extract(py)?)
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
