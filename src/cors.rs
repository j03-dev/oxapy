use crate::{into_response::IntoResponse, response::Response, status::Status};
use pyo3::prelude::*;

#[derive(Clone, Debug)]
#[pyclass]
pub struct Cors {
    #[pyo3(get, set)]
    pub origins: Vec<String>,
    #[pyo3(get, set)]
    pub methods: Vec<String>,
    #[pyo3(get, set)]
    pub headers: Vec<String>,
    #[pyo3(get, set)]
    pub allow_credentials: bool,
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
    #[new]
    fn new() -> Self {
        Self::default()
    }

    fn __repr__(&self) -> String {
        format!("{:#?}", self.clone())
    }
}

impl IntoResponse for Cors {
    fn into_response(&self) -> PyResult<Response> {
        let mut response = Status::NO_CONTENT.into_response()?;
        self.apply_headers(&mut response);
        Ok(response)
    }
}

impl Cors {
    // Centralized method to apply CORS headers to any response
    pub fn apply_headers(&self, response: &mut Response) {
        response.header(
            "Access-Control-Allow-Origin".to_string(),
            self.origins.join(", "),
        );
        response.header(
            "Access-Control-Allow-Methods".to_string(),
            self.methods.join(", "),
        );
        response.header(
            "Access-Control-Allow-Headers".to_string(),
            self.headers.join(", "),
        );

        if self.allow_credentials {
            response.header(
                "Access-Control-Allow-Credentials".to_string(),
                "true".to_string(),
            );
        }

        response.header(
            "Access-Control-Max-Age".to_string(),
            self.max_age.to_string(),
        );
    }

    pub fn apply_to_response(&self, mut response: Response) -> PyResult<Response> {
        self.apply_headers(&mut response);
        Ok(response)
    }
}
