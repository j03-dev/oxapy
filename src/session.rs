use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use pyo3::{prelude::*, types::PyTuple, IntoPyObjectExt};
use rand::{distr::Alphanumeric, Rng};

use crate::IntoPyException;

type SessionData = HashMap<String, PyObject>;

pub fn generate_session_id() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

#[derive(Clone, Debug)]
#[pyclass]
pub struct Session {
    #[pyo3(get)]
    id: String,
    data: Arc<RwLock<SessionData>>,
    #[pyo3(get)]
    create_at: u64,
    last_accessed: Arc<Mutex<u64>>,
    modified: Arc<Mutex<bool>>,
}

#[pymethods]
impl Session {
    #[new]
    fn new(id: Option<String>) -> PyResult<Self> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .into_py_exception()?
            .as_secs();

        Ok(Self {
            id: id.unwrap_or_else(generate_session_id),
            data: Arc::new(RwLock::new(HashMap::new())),
            create_at: now,
            last_accessed: Arc::new(Mutex::new(now)),
            modified: Arc::new(Mutex::new(false)),
        })
    }

    fn get(&self, key: &str, py: Python<'_>) -> PyResult<PyObject> {
        *self.last_accessed.lock().into_py_exception()? = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .into_py_exception()?
            .as_secs();

        let data = self.data.read().into_py_exception()?;

        let value = data
            .get(key)
            .map(|value| value.clone_ref(py))
            .unwrap_or(py.None());

        Ok(value)
    }

    fn set(&self, key: &str, value: PyObject) -> PyResult<()> {
        let mut data = self.data.write().into_py_exception()?;
        data.insert(key.to_string(), value);
        *self.modified.lock().unwrap() = true;
        Ok(())
    }

    fn remove(&self, key: &str) -> PyResult<()> {
        let mut data = self.data.write().into_py_exception()?;
        if data.remove(key).is_some() {
            *self.modified.lock().into_py_exception()? = true;
        }
        Ok(())
    }

    fn clear(&self) -> PyResult<()> {
        let mut data = self.data.write().into_py_exception()?;
        if !data.is_empty() {
            data.clear();
            *self.modified.lock().into_py_exception()? = true;
        }
        Ok(())
    }

    fn keys(&self, py: Python<'_>) -> PyResult<PyObject> {
        let data = self.data.read().into_py_exception()?;
        let keys: Vec<String> = data.keys().cloned().collect();
        keys.into_py_any(py)
    }

    fn values(&self, py: Python<'_>) -> PyResult<PyObject> {
        let data = self.data.read().into_py_exception()?;
        let values: Vec<PyObject> = data.values().map(|v| v.clone_ref(py)).collect();
        values.into_py_any(py)
    }

    fn items(&self, py: Python<'_>) -> PyResult<PyObject> {
        let data = self.data.read().into_py_exception()?;
        let items: Vec<(String, PyObject)> = data
            .iter()
            .map(|(k, v)| (k.clone(), v.clone_ref(py)))
            .collect();
        items.into_py_any(py)
    }

    fn __contains__(&self, key: &str) -> PyResult<bool> {
        let data = self.data.read().into_py_exception()?;
        Ok(data.contains_key(key))
    }

    fn __iter__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<PyObject> {
        let keys = slf.keys(py)?;
        let iter_func = py.get_type::<PyTuple>().call_method1("__iter__", (keys,))?;
        iter_func.into_py_any(py)
    }

    fn __getitem__(&self, key: &str, py: Python<'_>) -> PyResult<PyObject> {
        let data = self.data.read().into_py_exception()?;
        match data.get(key) {
            Some(value) => Ok(value.clone_ref(py)),
            None => Err(pyo3::exceptions::PyKeyError::new_err(key.to_string())),
        }
    }

    fn __setitem__(&self, key: &str, value: PyObject) -> PyResult<()> {
        self.set(key, value)
    }

    fn __delitem__(&self, key: &str) -> PyResult<()> {
        let mut data = self.data.write().into_py_exception()?;
        if data.remove(key).is_some() {
            *self.modified.lock().into_py_exception()? = true;
            Ok(())
        } else {
            Err(pyo3::exceptions::PyKeyError::new_err(key.to_string()))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        *self.last_accessed.lock().into_py_exception()? = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .into_py_exception()?
            .as_secs();

        let data = self.data.read().into_py_exception()?;
        Ok(data.len())
    }

    fn __repr__(&self) -> String {
        format!("Session(id='{}')", self.id)
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

#[derive(Clone, Debug)]
#[pyclass]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, Arc<Session>>>>,
    #[pyo3(get, set)]
    pub cookie_name: String,
    #[pyo3(get, set)]
    cookie_max_age: Option<u64>,
    #[pyo3(get, set)]
    cookie_path: String,
    #[pyo3(get, set)]
    cookie_secure: bool,
    #[pyo3(get, set)]
    cookie_http_only: bool,
    #[pyo3(get, set)]
    cookie_same_site: String,
    #[pyo3(get, set)]
    expiry_seconds: Option<u64>,
}

#[pymethods]
impl SessionStore {
    #[new]
    #[pyo3(signature = (
        cookie_name = "session".to_string(),
        cookie_max_age = None,
        cookie_path = "/".to_string(),
        cookie_secure = false,
        cookie_http_only = true,
        cookie_same_site = "Lax".to_string(),
        expiry_seconds = Some(86400)
    ))]
    fn new(
        cookie_name: String,
        cookie_max_age: Option<u64>,
        cookie_path: String,
        cookie_secure: bool,
        cookie_http_only: bool,
        cookie_same_site: String,
        expiry_seconds: Option<u64>,
    ) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            cookie_name,
            cookie_max_age,
            cookie_path,
            cookie_secure,
            cookie_http_only,
            cookie_same_site,
            expiry_seconds,
        }
    }

    pub fn get_session(&self, session_id: Option<String>) -> PyResult<Session> {
        let mut sessions = self.sessions.write().into_py_exception()?;

        if let Some(id) = session_id {
            if let Some(session) = sessions.get(&id) {
                *session.last_accessed.lock().unwrap() = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .into_py_exception()?
                    .as_secs();

                return Ok(session.as_ref().clone());
            }
        }

        let session = Session::new(None)?;
        let id = session.id.clone();
        sessions.insert(id, Arc::new(session.clone()));

        Ok(session)
    }

    fn clear_session(&self, session_id: &str) -> PyResult<bool> {
        let mut sessions = self.sessions.write().into_py_exception()?;
        Ok(sessions.remove(session_id).is_some())
    }

    fn session_count(&self) -> PyResult<usize> {
        let sessions = self.sessions.read().into_py_exception()?;
        Ok(sessions.len())
    }

    pub fn get_cookie_header(&self, session: &Session) -> String {
        let mut header = format!(
            "{}={}; Path={}",
            self.cookie_name, session.id, self.cookie_path
        );

        if let Some(max_age) = self.cookie_max_age {
            header.push_str(&format!("; Max-Age={}", max_age));
        }

        if self.cookie_secure {
            header.push_str("; Secure");
        }

        if self.cookie_http_only {
            header.push_str("; HttpOnly");
        }

        header.push_str(&format!("; SameSite={}", self.cookie_same_site));

        header
    }
}
