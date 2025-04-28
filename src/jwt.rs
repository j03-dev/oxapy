use jsonwebtoken::{
    decode, encode, errors::Error as JWTError, Algorithm, DecodingKey, EncodingKey, Header,
    Validation,
};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: u64,
    #[serde(flatten)]
    payload: serde_json::Value,
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("JWT error: {0}")]
    Jwt(#[from] JWTError),
    #[error("System time error: {0}")]
    Time(#[from] std::time::SystemTimeError),
    #[error("Invalid JWT payload")]
    InvalidPayload,
}

impl std::convert::From<JwtError> for PyErr {
    fn from(err: JwtError) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyException, _>(format!("{}", err))
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Jwt {
    secret: String,
    algorithm: Algorithm,
    expiration: Duration,
}

#[pymethods]
impl Jwt {
    #[new]
    #[pyo3(signature = (secret, algorithm="HS256", expiration_minutes=60))]
    pub fn new(secret: String, algorithm: &str, expiration_minutes: u64) -> PyResult<Self> {
        let algorithm = match algorithm {
            "HS256" => Algorithm::HS256,
            "HS384" => Algorithm::HS384,
            "HS512" => Algorithm::HS512,
            "RS256" => Algorithm::RS256,
            "RS384" => Algorithm::RS384,
            "RS512" => Algorithm::RS512,
            "ES256" => Algorithm::ES256,
            "ES384" => Algorithm::ES384,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "Unsupported algorithm",
                ))
            }
        };

        Ok(Self {
            secret,
            algorithm,
            expiration: Duration::from_secs(expiration_minutes * 60),
        })
    }

    pub fn generate_token(&self, py: Python<'_>, claims: &Bound<'_, PyDict>) -> PyResult<String> {
        let json_module = PyModule::import(py, "json")?;
        let claims_json: String = json_module
            .call_method("dumps", (claims,), None)?
            .extract()?;

        let payload: serde_json::Value =
            serde_json::from_str(&claims_json).map_err(|_| JwtError::InvalidPayload)?;

        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| JwtError::Time(e))?
            .checked_add(self.expiration)
            .ok_or(JwtError::InvalidPayload)?
            .as_secs();

        let claims = Claims { exp, payload };

        encode(
            &Header::new(self.algorithm),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| JwtError::Jwt(e).into())
    }

    pub fn verify_token<'a>(&self, py: Python<'a>, token: &str) -> PyResult<Bound<'a, PyDict>> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::new(self.algorithm),
        )
        .map_err(|e| JwtError::Jwt(e))?;

        let dict = PyDict::new(py);
        if let serde_json::Value::Object(payload) = token_data.claims.payload {
            for (key, value) in payload {
                let py_value = match value {
                    serde_json::Value::Null => py.None(),
                    serde_json::Value::Bool(b) => b.into_py(py),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            i.into_py(py)
                        } else if let Some(f) = n.as_f64() {
                            f.into_py(py)
                        } else {
                            return Err(JwtError::InvalidPayload.into());
                        }
                    }
                    serde_json::Value::String(s) => s.into_py(py),
                    _ => return Err(JwtError::InvalidPayload.into()),
                };

                dict.set_item(key, py_value)?;
            }
        }
        Ok(dict)
    }

    #[getter]
    fn expiration(&self) -> u64 {
        self.expiration.as_secs()
    }

    #[getter]
    fn algorithm(&self) -> String {
        format!("{:?}", self.algorithm)
    }
}

pub fn jwt_submodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Jwt>()?;
    Ok(())
}
