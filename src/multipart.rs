use ahash::HashMap;
use futures_util::stream;
use hyper::body::Bytes;
use multer::Multipart;
use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};

use crate::IntoPyException;

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
/// @router.post("/upload")
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

pub struct MultiPart {
    pub fields: HashMap<String, String>,
    pub files: HashMap<String, File>,
}

pub async fn parse_mutltipart(content_type: &str, body_stream: Bytes) -> PyResult<MultiPart> {
    let mut fields = HashMap::default();
    let mut files = HashMap::default();

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
