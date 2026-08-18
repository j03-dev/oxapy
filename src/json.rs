use pyo3::{prelude::*, sync::PyOnceLock, types::PyDict};
use serde::{Deserialize, Serialize};

fn orjson(py: Python<'_>) -> PyResult<&Py<PyModule>> {
    static ORJSON: PyOnceLock<Py<PyModule>> = PyOnceLock::new();
    ORJSON.get_or_try_init(py, || PyModule::import(py, "orjson").map(|m| m.into()))
}

#[inline]
pub fn dumps(data: &Bound<PyAny>) -> PyResult<Vec<u8>> {
    let py = data.py();
    let serialized_data = orjson(py)?.call_method1(py, "dumps", (data,))?;
    serialized_data.extract(py)
}

#[inline]
pub fn loads(data: &str, py: Python<'_>) -> PyResult<Py<PyDict>> {
    let deserialized_data = orjson(py)?.call_method1(py, "loads", (data,))?;
    Ok(deserialized_data.extract(py)?)
}

pub fn from_pydict2rstruct<T>(dict: &Bound<'_, PyAny>) -> PyResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    let json_bytes = dumps(dict)?;
    let value = serde_json::from_slice(&json_bytes)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok(value)
}

pub fn from_rstruct2pydict<T>(rstruct: T, py: Python<'_>) -> PyResult<Py<PyDict>>
where
    T: Serialize,
{
    let json_string = serde_json::to_string(&rstruct)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    loads(&json_string, py)
}
