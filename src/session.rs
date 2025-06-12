use std::{
    sync::{Arc, Mutex, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use ahash::HashMap;
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

/// Session storage for maintaining state between requests.
///
/// The Session class provides a dictionary-like interface for storing data
/// that persists across multiple requests from the same client.
///
/// Args:
///     id (str, optional): Custom session ID. If not provided, a random ID will be generated.
///
/// Returns:
///     Session: A new session instance.
///
/// Example:
/// ```python
/// # Sessions are typically accessed from the request object:
/// @router.get("/profile")
/// def profile(request):
///     session = request.session()
///     session["last_visit"] = "today"
///     return {"user_id": session.get("user_id")}
/// ```
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
    /// Create a new Session instance.
    ///
    /// Args:
    ///     id (str, optional): Custom session ID. If not provided, a random ID will be generated.
    ///
    /// Returns:
    ///     Session: A new session instance.
    ///
    /// Example:
    /// ```python
    /// # Manual session creation (normally handled by the framework)
    /// session = Session()
    /// ```
    #[new]
    fn new(id: Option<String>) -> PyResult<Self> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .into_py_exception()?
            .as_secs();

        Ok(Self {
            id: id.unwrap_or_else(generate_session_id),
            data: Arc::new(RwLock::new(HashMap::default())),
            create_at: now,
            last_accessed: Arc::new(Mutex::new(now)),
            modified: Arc::new(Mutex::new(false)),
        })
    }

    /// Get a value from the session by key.
    ///
    /// Args:
    ///     key (str): The key to look up in the session.
    ///
    /// Returns:
    ///     any: The value associated with the key, or None if the key doesn't exist.
    ///
    /// Example:
    /// ```python
    /// user_id = session.get("user_id")
    /// if user_id is not None:
    ///     # User is logged in
    /// ```
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

    /// Set a value in the session.
    ///
    /// Args:
    ///     key (str): The key to store the value under.
    ///     value (any): The value to store in the session.
    ///
    /// Returns:
    ///     None
    ///
    /// Example:
    /// ```python
    /// # Store user information in the session
    /// session.set("user_id", 123)
    /// session.set("is_admin", False)
    /// ```
    fn set(&self, key: &str, value: PyObject) -> PyResult<()> {
        let mut data = self.data.write().into_py_exception()?;
        data.insert(key.to_string(), value);
        *self.modified.lock().unwrap() = true;
        Ok(())
    }

    /// Remove a key-value pair from the session.
    ///
    /// Args:
    ///     key (str): The key to remove.
    ///
    /// Returns:
    ///     None
    ///
    /// Example:
    /// ```python
    /// # Log user out by removing their session data
    /// session.remove("user_id")
    /// session.remove("is_admin")
    /// ```
    fn remove(&self, key: &str) -> PyResult<()> {
        let mut data = self.data.write().into_py_exception()?;
        if data.remove(key).is_some() {
            *self.modified.lock().into_py_exception()? = true;
        }
        Ok(())
    }

    /// Remove all data from the session.
    ///
    /// Args:
    ///     None
    ///
    /// Returns:
    ///     None
    ///
    /// Example:
    /// ```python
    /// # Clear all session data (e.g., during logout)
    /// session.clear()
    /// ```
    fn clear(&self) -> PyResult<()> {
        let mut data = self.data.write().into_py_exception()?;
        if !data.is_empty() {
            data.clear();
            *self.modified.lock().into_py_exception()? = true;
        }
        Ok(())
    }

    /// Get all keys in the session.
    ///
    /// Args:
    ///     None
    ///
    /// Returns:
    ///     list: A list of all keys in the session.
    ///
    /// Example:
    /// ```python
    /// # Check what data is stored in the session
    /// for key in session.keys():
    ///     print(f"Session contains: {key}")
    /// ```
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
            _ => Err(pyo3::exceptions::PyKeyError::new_err(key.to_string())),
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

/// Manages sessions for the application.
///
/// The SessionStore maintains all active sessions and handles their serialization
/// and deserialization via cookies.
///
/// Args:
///     cookie_name (str, optional): Name of the cookie used for session tracking (default: "session").
///     cookie_max_age (int, optional): Max age of the cookie in seconds (default: None).
///     cookie_path (str, optional): Path for the cookie (default: "/").
///     cookie_secure (bool, optional): Whether the cookie should only be sent over HTTPS (default: False).
///     cookie_http_only (bool, optional): Whether the cookie is inaccessible to JavaScript (default: True).
///     cookie_same_site (str, optional): SameSite cookie policy ("Lax", "Strict", or "None") (default: "Lax").
///     expiry_seconds (int, optional): How long sessions should last in seconds (default: 86400 - one day).
///
/// Returns:
///     SessionStore: A new session store instance.
///
/// Example:
/// ```python
/// from oxapy import HttpServer, SessionStore
///
/// app = HttpServer(("127.0.0.1", 8000))
///
/// # Configure sessions with custom settings
/// store = SessionStore(
///     cookie_name="my_app_session",
///     cookie_secure=True,
///     expiry_seconds=3600  # 1 hour
/// )
/// app.session_store(store)
/// ```
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
    /// Create a new SessionStore.
    ///
    /// Args:
    ///     cookie_name (str, optional): Name of the cookie used for session tracking (default: "session").
    ///     cookie_max_age (int, optional): Max age of the cookie in seconds (default: None).
    ///     cookie_path (str, optional): Path for the cookie (default: "/").
    ///     cookie_secure (bool, optional): Whether the cookie should only be sent over HTTPS (default: False).
    ///     cookie_http_only (bool, optional): Whether the cookie is inaccessible to JavaScript (default: True).
    ///     cookie_same_site (str, optional): SameSite cookie policy ("Lax", "Strict", or "None") (default: "Lax").
    ///     expiry_seconds (int, optional): How long sessions should last in seconds (default: 86400 - one day).
    ///
    /// Returns:
    ///     SessionStore: A new session store instance.
    ///
    /// Example:
    /// ```python
    /// # Create a session store with default settings
    /// store = SessionStore()
    ///
    /// # Create a session store with custom settings
    /// secure_store = SessionStore(
    ///     cookie_name="secure_session",
    ///     cookie_secure=True,
    ///     cookie_same_site="Strict"
    /// )
    /// ```
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
            sessions: Arc::new(RwLock::new(HashMap::default())),
            cookie_name,
            cookie_max_age,
            cookie_path,
            cookie_secure,
            cookie_http_only,
            cookie_same_site,
            expiry_seconds,
        }
    }

    /// Get a session by ID or create a new one if not found.
    ///
    /// Args:
    ///     session_id (str, optional): The session ID to look up.
    ///
    /// Returns:
    ///     Session: The existing session if found, or a new session otherwise.
    ///
    /// Note:
    ///     This method is primarily used internally by the framework.
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

    /// Remove a session from the store.
    ///
    /// Args:
    ///     session_id (str): The ID of the session to remove.
    ///
    /// Returns:
    ///     bool: True if the session was found and removed, False otherwise.
    ///
    /// Example:
    /// ```python
    /// # Clear a specific session
    /// session_store.clear_session("abcd1234")
    /// ```
    fn clear_session(&self, session_id: &str) -> PyResult<bool> {
        let mut sessions = self.sessions.write().into_py_exception()?;
        Ok(sessions.remove(session_id).is_some())
    }

    /// Get the total number of active sessions.
    ///
    /// Args:
    ///     None
    ///
    /// Returns:
    ///     int: The number of active sessions in the store.
    ///
    /// Example:
    /// ```python
    /// # Check how many active sessions exist
    /// count = session_store.session_count()
    /// print(f"Active sessions: {count}")
    /// ```
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
