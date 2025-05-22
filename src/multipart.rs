use std::collections::HashMap;

use futures_util::stream;
use hyper::body::Bytes;
use multer::Multipart;
use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};

use crate::IntoPyException;

#[derive(Clone, Debug)]
#[pyclass]
pub struct File {
    #[pyo3(get)]
    pub name: Option<String>,
    #[pyo3(set, get)]
    pub content_type: Option<String>,
    pub data: Bytes,
}

#[pymethods]
impl File {
    fn content<'py>(&'py self, py: Python<'py>) -> Bound<'py, PyBytes> {
        let data = &self.data.to_vec()[..];
        PyBytes::new(py, data)
    }

    fn save(&self, path: String) -> PyResult<()> {
        std::fs::write(path, &self.data)?;
        Ok(())
    }
}

pub struct MultiPart {
    pub fields: HashMap<String, String>,
    pub files: HashMap<String, File>,
}

pub async fn parse_mutltipart(content_type: &str, body_stream: Bytes) -> PyResult<MultiPart> {
    let mut fields = HashMap::new();
    let mut files = HashMap::new();

    let boundary = content_type
        .split("boundary=")
        .nth(1)
        .map(|b| b.trim().to_string())
        .ok_or_else(|| PyValueError::new_err("Boundary not found in Content-Type header"))?;

    let stream = stream::once(async { Result::<Bytes, std::io::Error>::Ok(body_stream) });
    let mut multipart = Multipart::new(stream, boundary);

    while let Some(mut field) = multipart.next_field().await.into_py_exception()? {
        if field.content_type().is_some() || field.file_name().is_some() {
            let file_name = field.file_name().map(String::from);
            let content_type = field.content_type().map(|ct| ct.to_string());
            let mut file_data = Vec::new();
            while let Some(chunk) = field.chunk().await.into_py_exception()? {
                file_data.extend_from_slice(&chunk);
            }
            let file_bytes = Bytes::from(file_data);
            let file_obj = File {
                name: file_name,
                content_type,
                data: file_bytes,
            };
            files.insert(field.name().unwrap_or_default().to_string(), file_obj);
        } else {
            let field_name = field.name().unwrap_or_default().to_string();
            let field_value = field.text().await.into_py_exception()?;
            fields.insert(field_name, field_value);
        }
    }

    Ok(MultiPart { fields, files })
}
