use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use pyo3::exceptions::PyException;
use pyo3::types::PyDict;
use pyo3::{exceptions::PyValueError, impl_exception_boilerplate, prelude::*};
use pyo3_stub_gen::derive::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::exceptions::IntoPyException;
use crate::json::Wrap;

/// Base class for all JWT related exceptions.
#[gen_stub_pyclass]
#[pyclass(subclass, extends=PyException, module="oxapy.jwt")]
#[repr(transparent)]
pub struct JwtError(Py<PyAny>);

impl_exception_boilerplate!(JwtError);

#[gen_stub_pymethods]
#[pymethods]
impl JwtError {
    #[new]
    fn new(e: Py<PyAny>) -> JwtError {
        Self(e)
    }
}

/// Occurs when there's an error during JWT encoding.
#[gen_stub_pyclass]
#[pyclass(extends=JwtError, module="oxapy.jwt")]
pub struct JwtEncodingError;

impl_exception_boilerplate!(JwtEncodingError);
extend_exception!(JwtEncodingError, JwtError);

/// Occurs when there's an error during JWT decoding/verification.
#[gen_stub_pyclass]
#[pyclass(extends=JwtError, module="oxapy.jwt")]
pub struct JwtDecodingError;

impl_exception_boilerplate!(JwtDecodingError);
extend_exception!(JwtDecodingError, JwtError);

/// Occurs when the JWT algorithm is invalid or not supported.
#[gen_stub_pyclass]
#[pyclass(extends=JwtError, module="oxapy.jwt")]
pub struct JwtInvalidAlgorithm;
impl_exception_boilerplate!(JwtInvalidAlgorithm);
extend_exception!(JwtInvalidAlgorithm, JwtError);

/// Occurs when a JWT claim is invalid (e.g., wrong format).
#[gen_stub_pyclass]
#[pyclass(extends=JwtError, module="oxapy.jwt")]
pub struct JwtInvalidClaim;
impl_exception_boilerplate!(JwtInvalidClaim);
extend_exception!(JwtInvalidClaim, JwtError);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    exp: u64,
    sub: Option<String>,
    iss: Option<String>,
    aud: Option<String>,
    nbf: Option<u64>,

    #[serde(flatten)]
    extra: Value,
}

/// Python class for generating and verifying JWT tokens
#[gen_stub_pyclass]
#[pyclass(module = "oxapy.jwt")]
#[derive(Clone)]
pub struct Jwt {
    secret: String,
    algorithm: Algorithm,
}

#[gen_stub_pymethods]
#[pymethods]
impl Jwt {
    /// Create a new JWT
    ///
    /// Args:
    ///     secret (str): Secret key used for signing tokens
    ///     algorithm (str): JWT algorithm to use (default: "HS256")
    ///
    /// Returns:
    ///     Jwt: A new Jwt instance
    ///
    /// Raises:
    ///     Exception: If the algorithm is not supported or secret is invalid
    ///
    /// Example:
    /// ```python
    /// from oxapy import jwt
    ///
    /// jwt_handler = jwt.Jwt(secret="mysecret", algorithm="HS256")
    /// ```
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
    ///
    /// Example:
    /// ```python
    /// from oxapy import jwt, Router, post
    ///
    /// jwt_handler = jwt.Jwt(secret="mysecret", algorithm="HS256")
    /// router = Router()
    ///
    /// @post("/login")
    /// def login(request):
    ///     # Authenticate user...
    ///     claims = {
    ///         "exp": 3600,  # seconds from now
    ///         "sub": "user123",  # subject (optional)
    ///         "iss": "myapp",    # issuer (optional)
    ///         "aud": "webapp",   # audience (optional)
    ///         "nbf": 1234567890  # not before timestamp (optional)
    ///     }
    ///     token = jwt_handler.generate_token(claims)
    ///     return {"token": token}
    /// ```
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

        let Wrap::<Claims>(claims) = claims.try_into()?;

        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| JwtError::new_err(e.to_string()))?;

        Ok(token)
    }

    /// Verify the integrity of the JWT token
    ///
    /// Args:
    ///     token: A JWT token String
    ///
    /// Returns:
    ///     Return Dictionary: the claims that you use to generate the token
    ///
    /// Raises:
    ///     JwtError: if token was expired or not valid token
    ///
    /// Example:
    /// ```python
    /// from oxapy import jwt, Router, exceptions, get
    ///
    /// jwt_handler = jwt.Jwt(secret="mysecret", algorithm="HS256")
    /// router = Router()
    ///
    /// @get("/protected")
    /// def protected_route(request):
    ///     token = request.headers.get("Authorization", "").replace("Bearer ", "")
    ///     try:
    ///         claims = jwt_handler.verify_token(token)
    ///         return {"user_id": claims["sub"], "message": "Access granted"}
    ///     except jwt.JwtDecodingError:
    ///         raise exceptions.UnauthorizedError("Invalid or expired token")
    /// ```
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
        Wrap(token_data.claims).try_into()
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
