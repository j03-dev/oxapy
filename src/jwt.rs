use crate::{json, IntoPyException, Wrap};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
        // Validate secret key
        if secret.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Secret key cannot be empty",
            ));
        }

        let algorithm = match algorithm {
            "HS256" => Algorithm::HS256,
            "HS384" => Algorithm::HS384,
            "HS512" => Algorithm::HS512,
            "RS256" | "RS384" | "RS512" | "ES256" | "ES384" => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "Asymmetric algorithms are not yet supported – use HS256/384/512",
                ))
            }
            &_ => todo!(),
        };

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

        let claims = serde_json::json!(token_data.claims).to_string();
        json::loads(&claims)
    }
}

pub fn jwt_submodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let jwt = PyModule::new(m.py(), "jwt")?;
    jwt.add_class::<Jwt>()?;
    m.add_submodule(&jwt)
}
