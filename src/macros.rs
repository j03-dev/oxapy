#[macro_export]
macro_rules! extend_exception {
    ($name:ident, $extend:ident) => {
        #[pyo3_stub_gen::derive::gen_stub_pymethods]
        #[pyo3::prelude::pymethods]
        impl $name {
            #[new]
            fn new(e: pyo3::Py<pyo3::PyAny>) -> ($name, $extend) {
                ($name, $extend(e))
            }
        }
    };
}
