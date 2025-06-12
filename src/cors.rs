use crate::{response::Response, status::Status};
use pyo3::prelude::*;

/// Cross-Origin Resource Sharing (CORS) configuration.
///
/// This class allows you to configure CORS headers for your server to control
/// which domains can access your API and what methods they can use.
///
/// Args:
///     None
///
/// Returns:
///     Cors: A new CORS configuration with default settings.
///
/// Example:
/// ```python
/// from oxapy import HttpServer, Cors
///
/// app = HttpServer(("127.0.0.1", 8000))
///
/// # Set up CORS with custom configuration
/// cors = Cors()
/// cors.origins = ["https://example.com", "https://app.example.com"]
/// cors.methods = ["GET", "POST", "OPTIONS"]
/// cors.headers = ["Content-Type", "Authorization"]
///
/// app.cors(cors)
/// ```
#[derive(Clone, Debug)]
#[pyclass]
pub struct Cors {
    /// List of allowed origins, default is ["*"] (all origins)
    #[pyo3(get, set)]
    pub origins: Vec<String>,
    /// List of allowed HTTP methods, default includes common methods
    #[pyo3(get, set)]
    pub methods: Vec<String>,
    /// List of allowed HTTP headers, default includes common headers
    #[pyo3(get, set)]
    pub headers: Vec<String>,
    /// Whether to allow credentials (cookies, authorization headers), default is true
    #[pyo3(get, set)]
    pub allow_credentials: bool,
    /// Maximum age of preflight requests in seconds, default is 86400 (1 day)
    #[pyo3(get, set)]
    pub max_age: u32,
}

impl Default for Cors {
    fn default() -> Self {
        Self {
            origins: vec!["*".to_string()],
            methods: vec!["GET, POST, PUT, DELETE, PATCH, OPTIONS".to_string()],
            headers: vec!["Content-Type, Authorization, X-Requested-With, Accept".to_string()],
            allow_credentials: true,
            max_age: 86400,
        }
    }
}

#[pymethods]
impl Cors {
    /// Create a new CORS configuration with default settings.
    ///
    /// Returns:
    ///     Cors: A new CORS configuration with default values.
    ///
    /// Example:
    /// ```python
    /// # Create CORS with default configuration (allows all origins)
    /// cors = Cors()
    ///
    /// # Customize CORS settings
    /// cors.origins = ["https://example.com"]
    /// cors.allow_credentials = False
    /// ```
    #[new]
    fn new() -> Self {
        Self::default()
    }

    /// Return a string representation of the CORS configuration.
    ///
    /// Returns:
    ///     str: A debug string showing the CORS configuration.
    fn __repr__(&self) -> String {
        format!("{:#?}", self.clone())
    }
}

impl From<Cors> for Response {
    fn from(val: Cors) -> Self {
        let mut response = Status::NO_CONTENT.into();
        val.apply_headers(&mut response);
        response
    }
}

impl Cors {
    /// Apply CORS headers to a response.
    ///
    /// This is an internal method used to add all configured CORS headers
    /// to an existing response.
    ///
    /// Args:
    ///     response (Response): The response to modify.
    ///
    /// Returns:
    ///     None
    pub fn apply_headers(&self, response: &mut Response) {
        response.insert_header("Access-Control-Allow-Origin", self.origins.join(", "));
        response.insert_header("Access-Control-Allow-Methods", self.methods.join(", "));
        response.insert_header("Access-Control-Allow-Headers", self.headers.join(", "));
        if self.allow_credentials {
            response.insert_header("Access-Control-Allow-Credentials", "true".to_string());
        }
        response.insert_header("Access-Control-Max-Age", self.max_age.to_string());
    }

    /// Apply CORS headers to a response and return the modified response.
    ///
    /// Args:
    ///     response (Response): The response to modify with CORS headers.
    ///
    /// Returns:
    ///     Response: The modified response with CORS headers.
    pub fn apply_to_response(&self, mut response: Response) -> PyResult<Response> {
        self.apply_headers(&mut response);
        Ok(response)
    }
}
