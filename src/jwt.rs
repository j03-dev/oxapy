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
    iss: Option<String>,
    sub: Option<String>,
    aud: Option<String>,
    exp: u64,
    nbf: Option<u64>,
    iat: Option<u64>,
    jti: Option<String>,

    #[serde(flatten)]
    extra: serde_json::Value,
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

#[pyclass(name = "JwtManager")]
#[derive(Clone)]
pub struct JwtManager {
    secret: String,
    algorithm: Algorithm,
    expiration: Duration,
}

#[pymethods]
impl JwtManager {
    #[new]
    #[pyo3(signature = (secret, algorithm="HS256", expiration_minutes=60))]
    pub fn new(secret: String, algorithm: &str, expiration_minutes: u64) -> PyResult<Self> {
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

    pub fn generate_token(&self, py: Python<'_>, claims: &Bound<'_, PyDict>) -> PyResult<String> {
        let json_module = PyModule::import(py, "json")?;
        let claims_json: String = json_module
            .call_method("dumps", (claims,), None)?
            .extract()?;

        let raw_payload: serde_json::Value =
            serde_json::from_str(&claims_json).map_err(|_| JwtError::InvalidPayload)?;

        let mut standard = Claims {
            iss: None,
            sub: None,
            aud: None,
            exp: 0,
            nbf: None,
            iat: None,
            jti: None,
            extra: serde_json::Value::Null,
        };

        let mut extras = serde_json::Map::new();

        if let serde_json::Value::Object(map) = raw_payload {
            for (k, v) in map {
                match k.as_str() {
                    "iss" | "sub" | "aud" | "jti" => {
                        if let serde_json::Value::String(s) = v {
                            match k.as_str() {
                                "iss" => standard.iss = Some(s),
                                "sub" => standard.sub = Some(s),
                                "aud" => standard.aud = Some(s),
                                "jti" => standard.jti = Some(s),
                                _ => {}
                            }
                        } else {
                            return Err(JwtError::InvalidPayload.into());
                        }
                    }
                    "nbf" | "iat" => {
                        if let serde_json::Value::Number(n) = v {
                            if let Some(u) = n.as_u64() {
                                match k.as_str() {
                                    "nbf" => standard.nbf = Some(u),
                                    "iat" => standard.iat = Some(u),
                                    _ => {}
                                }
                            } else {
                                return Err(JwtError::InvalidPayload.into());
                            }
                        } else {
                            return Err(JwtError::InvalidPayload.into());
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
            .map_err(JwtError::Time)?;
        standard.iat.get_or_insert(now.as_secs());

        let exp = now
            .checked_add(self.expiration)
            .ok_or(JwtError::InvalidPayload)?
            .as_secs();

        standard.exp = exp;
        standard.extra = serde_json::Value::Object(extras);

        encode(
            &Header::new(self.algorithm),
            &standard,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| JwtError::Jwt(e).into())
    }

    pub fn verify_token<'a>(&self, py: Python<'a>, token: &str) -> PyResult<Bound<'a, PyDict>> {
        let mut validation = Validation::new(self.algorithm);
        validation.required_spec_claims = ["exp"].iter().map(|&s| s.to_string()).collect();

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(JwtError::Jwt)?;

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

        if let serde_json::Value::Object(extra) = token_data.claims.extra {
            for (key, value) in extra {
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

pub fn jwt_submodule(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let jwt = PyModule::new(parent_module.py(), "jwt")?;
    jwt.add_class::<JwtManager>()?;
    parent_module.add_submodule(&jwt)
}
