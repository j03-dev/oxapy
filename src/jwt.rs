use crate::{IntoPyException, Wrap};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::types::PyDict;
use pyo3::{exceptions::PyValueError, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

create_exception!("jwt", JwtError, PyException);
create_exception!("jwt", JwtEncodingError, JwtError);
create_exception!("jwt", JwtDecodingError, JwtError);
create_exception!("jwt", JwtInvalidAlgorithm, JwtError);
create_exception!("jwt", JwtInvalidClaim, JwtError);

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: u64,
    sub: Option<String>,
    iss: Option<String>,
    aud: Option<String>,
    nbf: Option<u64>,

    #[serde(flatten)]
    extra: Value,
}

#[pyclass]
/// Python class for generating and verifying JWT tokens
#[derive(Clone)]
pub struct Jwt {
    secret: String,
    algorithm: Algorithm,
}

#[pymethods]
impl Jwt {
    /// Create a new JWT manager
    ///
    /// Args:
    ///     secret: Secret key used for signing tokens
    ///     algorithm: JWT algorithm to use (default: "HS256")
    ///
    /// Returns:
    ///     A new JwtManager instance
    ///
    /// Raises:
    ///     ValueError: If the algorithm is not supported or secret is invalid

    #[new]
    #[pyo3(signature = (secret, algorithm="HS256"))]
    pub fn new(secret: String, algorithm: &str) -> PyResult<Self> {
        if secret.is_empty() {
            return Err(PyValueError::new_err("Secret key cannot be empty"));
        }

        let algorithm = Algorithm::from_str(algorithm).into_py_exception()?;

        Ok(Self { secret, algorithm })
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
            .map(|exp| {
                exp.extract::<u64>()
                    .map_err(|_| JwtError::new_err("Invalid `exp` format"))
            })
            .transpose()?
            .unwrap_or(60);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| JwtError::new_err(e.to_string()))?;

        let exp = now
            .checked_add(Duration::from_secs(expiration))
            .ok_or_else(|| JwtError::new_err("Failed to compute expiration"))?;
        claims.set_item("exp", exp.as_secs())?;

        let Wrap::<Claims>(claims) = claims.into();

        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| JwtError::new_err(e.to_string()))?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> PyResult<Py<PyDict>> {
        let token_data = jsonwebtoken::decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::new(self.algorithm),
        )
        .map_err(|e| match e.kind() {
            ErrorKind::ExpiredSignature => JwtDecodingError::new_err("Token has expired"),
            ErrorKind::InvalidToken => JwtDecodingError::new_err("Invalid token structure"),
            ErrorKind::InvalidIssuer => JwtDecodingError::new_err("Invalid issuer"),
            ErrorKind::InvalidAudience => JwtDecodingError::new_err("Invalid audience"),
            ErrorKind::InvalidAlgorithm => JwtInvalidAlgorithm::new_err("Algorithm mismatch"),
            _ => JwtDecodingError::new_err(format!("JWT decoding error: {e}")),
        })?;
        Ok(Wrap(token_data.claims).into())
    }
}

pub fn jwt_submodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let jwt = PyModule::new(m.py(), "jwt")?;
    jwt.add_class::<Jwt>()?;
    jwt.add("JwtError", m.py().get_type::<JwtError>())?;
    jwt.add("JwtEncodingError", m.py().get_type::<JwtEncodingError>())?;
    jwt.add("JwtDecodingError", m.py().get_type::<JwtDecodingError>())?;
    jwt.add(
        "JwtInvalidAlgorithm",
        m.py().get_type::<JwtInvalidAlgorithm>(),
    )?;
    jwt.add("JwtInvalidClaim", m.py().get_type::<JwtInvalidClaim>())?;
    m.add_submodule(&jwt)
}
