use crate::json;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use pyo3::exceptions::PyException;
use pyo3::types::PyDict;
use pyo3::{create_exception, prelude::*};
use pyo3::{IntoPyObjectExt, PyObject};
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

create_exception!(jwt, JwtError, PyException, "JWT error");
create_exception!(jwt, TimeError, PyException, "System time error");
create_exception!(jwt, InvalidPayload, PyException, "Invalid JWT payload");

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
    pub fn generate_token(&self, _py: Python<'_>, claims: &Bound<'_, PyDict>) -> PyResult<String> {
        let claims_obj: PyObject = claims.to_owned().into();
        let claims_json = json::dumps(&claims_obj)?;

        let raw_payload: Value = serde_json::from_str(&claims_json)
            .map_err(|err| InvalidPayload::new_err(err.to_string()))?;

        let mut standard = Claims {
            iss: None,
            sub: None,
            aud: None,
            exp: 0,
            nbf: None,
            iat: None,
            jti: None,
            extra: Value::Null,
        };

        let mut extras = serde_json::Map::new();

        if let Value::Object(map) = raw_payload {
            for (k, v) in map {
                match k.as_str() {
                    "iss" | "sub" | "aud" | "jti" => {
                        if let Value::String(s) = v {
                            match k.as_str() {
                                "iss" => standard.iss = Some(s),
                                "sub" => standard.sub = Some(s),
                                "aud" => standard.aud = Some(s),
                                "jti" => standard.jti = Some(s),
                                _ => {}
                            }
                        } else {
                            return Err(InvalidPayload::new_err(
                                "['iss', 'sub', 'aud', 'jti'] should be a string",
                            ));
                        }
                    }
                    "nbf" | "iat" => {
                        if let Value::Number(n) = v {
                            if let Some(u) = n.as_u64() {
                                match k.as_str() {
                                    "nbf" => standard.nbf = Some(u),
                                    "iat" => standard.iat = Some(u),
                                    _ => {}
                                }
                            } else {
                                return Err(InvalidPayload::new_err("only real number"));
                            }
                        } else {
                            return Err(InvalidPayload::new_err(
                                "['iss', 'sub', 'aud', 'jti'] should be a string",
                            ));
                        }
                    }
                    "exp" => continue,
                    _ => {
                        extras.insert(k, v);
                    }
                }
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| TimeError::new_err(err.to_string()))?;
        standard.iat.get_or_insert(now.as_secs());

        let exp = now
            .checked_add(self.expiration)
            .ok_or(InvalidPayload::new_err("exipired"))?
            .as_secs();

        standard.exp = exp;
        standard.extra = Value::Object(extras);

        encode(
            &Header::new(self.algorithm),
            &standard,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| JwtError::new_err(e.to_string()))
    }

    pub fn verify_token<'a>(&self, py: Python<'a>, token: &str) -> PyResult<Bound<'a, PyDict>> {
        let mut validation = Validation::new(self.algorithm);
        validation.required_spec_claims = ["exp"].iter().map(|&s| s.to_string()).collect();

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|err| JwtError::new_err(err.to_string()))?;

        let dict = PyDict::new(py);

        if let Some(iss) = token_data.claims.iss {
            dict.set_item("iss", iss)?;
        }
        if let Some(sub) = token_data.claims.sub {
            dict.set_item("sub", sub)?;
        }
        if let Some(aud) = token_data.claims.aud {
            dict.set_item("aud", aud)?;
        }
        if let Some(nbf) = token_data.claims.nbf {
            dict.set_item("nbf", nbf)?;
        }
        if let Some(iat) = token_data.claims.iat {
            dict.set_item("iat", iat)?;
        }
        if let Some(jti) = token_data.claims.jti {
            dict.set_item("jti", jti)?;
        }
        dict.set_item("exp", token_data.claims.exp)?;

        if let Value::Object(extra) = token_data.claims.extra {
            for (key, value) in extra {
                let py_value = match value {
                    Value::Null => py.None(),
                    Value::Bool(b) => b.into_py_any(py)?,
                    Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            i.into_py_any(py)?
                        } else if let Some(f) = n.as_f64() {
                            f.into_py_any(py)?
                        } else {
                            return Err(InvalidPayload::new_err(""));
                        }
                    }
                    Value::String(s) => s.into_py_any(py)?,
                    _ => return Err(InvalidPayload::new_err("")),
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
    let jwt = PyModule::new(m.py(), "jwt")?;
    jwt.add_class::<Jwt>()?;
    jwt.add("JwtError", m.py().get_type::<JwtError>())?;
    jwt.add("TimeError", m.py().get_type::<TimeError>())?;
    jwt.add("InvalidPyload", m.py().get_type::<InvalidPayload>())?;
    m.add_submodule(&jwt)
}
