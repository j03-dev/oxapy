use crate::{IntoPyException, Wrap};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use pyo3::types::PyDict;
use pyo3::{exceptions::PyValueError, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: Option<String>,
    sub: Option<String>,
    aud: Option<String>,
    exp: u64,
    nbf: Option<u64>,
    iat: Option<u64>,
    jti: Option<String>,

    #[serde(flatten)]
    extra: Value,
}

#[pyclass]
/// Python class for generating and verifying JWT tokens
#[derive(Clone)]
pub struct Jwt {
    secret: String,
    algorithm: Algorithm,
    expiration: Duration,
}

#[pymethods]
impl Jwt {
    /// Create a new JWT manager
    ///
    /// Args:
    ///     secret: Secret key used for signing tokens
    ///     algorithm: JWT algorithm to use (default: "HS256")
    ///     expiration_minutes: Token expiration time in minutes (default: 60)
    ///
    /// Returns:
    ///     A new JwtManager instance
    ///
    /// Raises:
    ///     ValueError: If the algorithm is not supported or secret is invalid

    #[new]
    #[pyo3(signature = (secret, algorithm="HS256", expiration_minutes=60))]
    pub fn new(secret: String, algorithm: &str, expiration_minutes: u64) -> PyResult<Self> {
        if secret.is_empty() {
            return Err(PyValueError::new_err("Secret key cannot be empty"));
        }

        let algorithm = Algorithm::from_str(algorithm).into_py_exception()?;

        Ok(Self {
            secret,
            algorithm,
            expiration: Duration::from_secs(expiration_minutes * 60),
        })
    }

    /// Generate a JWT token with the given claims
    ///
    /// Args:
    ///     claims: A dictionary of claims to include in the token
    ///
    /// Returns:
    ///     JWT token string
    ///
    /// Raises:
    ///     Exception: If claims cannot be serialized or the token cannot be generated
    pub fn generate_token(&self, claims: Bound<'_, PyDict>) -> PyResult<String> {
        let expiration = claims
            .get_item("exp")?
            .map(|exp| Duration::from_secs(exp.extract::<u64>().unwrap() * 60))
            .unwrap_or(self.expiration);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .into_py_exception()?;

        if !claims.contains("iat")? {
            claims.set_item("iat", now.as_secs())?;
        }

        let exp = now.checked_add(expiration).unwrap();
        claims.set_item("exp", exp.as_secs())?;

        let Wrap::<Claims>(claims) = claims.into();

        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .into_py_exception()?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> PyResult<Py<PyDict>> {
        let token_data = jsonwebtoken::decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::new(self.algorithm),
        )
        .into_py_exception()?;

        Ok(Wrap(token_data.claims).into())
    }
}

pub fn jwt_submodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let jwt = PyModule::new(m.py(), "jwt")?;
    jwt.add_class::<Jwt>()?;
    m.add_submodule(&jwt)
}
