use crate::IntoPyException;
use ahash::HashMap;
use futures_util::stream;
use hyper::body::Bytes;
use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};
use pyo3_stub_gen::derive::*;

/// Represents an uploaded file in a multipart/form-data request.
///
/// The File class provides access to uploaded file data, including the file name,
/// content type, and binary content. It also allows saving the file to disk.
///
/// Args:
///     None (Files are created internally by the framework)
///
/// Returns:
///     File: A file object containing the uploaded data.
///
/// Example:
/// ```python
/// from oxapy import post
///
/// @post("/upload")
/// def upload_handler(request):
///     if request.files:
///         image = request.files.get("profile_image")
///         if image:
///             # Access file properties
///             filename = image.name
///             content_type = image.content_type
///             # Save the file
///             image.save(f"uploads/{filename}")
///             return {"status": "success", "filename": filename}
///     return {"status": "error", "message": "No file uploaded"}
/// ```
#[derive(Clone, Debug)]
#[gen_stub_pyclass]
#[pyclass]
pub struct File {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(set, get)]
    pub content_type: String,
    pub data: Bytes,
}

impl File {
    fn new(name: String, content_type: String, data: Bytes) -> Self {
        Self {
            name,
            content_type,
            data,
        }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl File {
    /// Get the file content as bytes.
    ///
    /// Args:
    ///     None
    ///
    /// Returns:
    ///     bytes: The file content as a Python bytes object.
    ///
    /// Example:
    /// ```python
    /// file_bytes = uploaded_file.content()
    /// file_size = len(file_bytes)
    /// ```
    fn content<'py>(&'py self, py: Python<'py>) -> Bound<'py, PyBytes> {
        let data = &self.data.to_vec()[..];
        PyBytes::new(py, data)
    }

    /// Save the file content to disk.
    ///
    /// Args:
    ///     path (str): The path where the file should be saved.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     Exception: If the file cannot be written to disk.
    ///
    /// Example:
    /// ```python
    /// # Save the uploaded file
    /// if "profile_image" in request.files:
    ///     image = request.files["profile_image"]
    ///     image.save(f"uploads/{image.name}")
    /// ```
    fn save(&self, path: String) -> PyResult<()> {
        std::fs::write(path, &self.data)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct Multipart {
    pub fields: HashMap<String, String>,
    pub files: HashMap<String, File>,
}

impl Multipart {
    async fn parse_field(&mut self, field: multer::Field<'_>) -> PyResult<()> {
        let name = field.name().unwrap_or_default().to_string();
        let value = field.text().await.into_py_exception()?;
        self.fields.insert(name, value);
        Ok(())
    }

    async fn parse_file(&mut self, mut field: multer::Field<'_>) -> PyResult<()> {
        let name = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();

        let mut data = Vec::new();
        while let Some(chunk) = field.chunk().await.into_py_exception()? {
            data.extend_from_slice(&chunk);
        }

        let file = File::new(name, content_type, data.into());

        self.files
            .insert(field.name().unwrap_or_default().to_string(), file);

        Ok(())
    }
}

pub async fn parse_multipart(content_type: &str, body_stream: Bytes) -> PyResult<Multipart> {
    let boundary = content_type
        .split("boundary=")
        .nth(1)
        .map(|b| b.trim().to_string())
        .ok_or_else(|| PyValueError::new_err("Boundary not found in Content-Type header"))?;

    let stream = stream::once(async { Result::<Bytes, std::io::Error>::Ok(body_stream) });
    let mut multer_multipart = multer::Multipart::new(stream, boundary);
    let mut multipart = Multipart::default();

    while let Some(field) = multer_multipart.next_field().await.into_py_exception()? {
        match (field.file_name(), field.content_type()) {
            (Some(_), Some(_)) => multipart.parse_file(field).await?,
            _ => multipart.parse_field(field).await?,
        }
    }

    Ok(multipart)
}
