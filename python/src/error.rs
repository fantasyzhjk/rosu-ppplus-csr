use pyo3::{create_exception, exceptions::PyException, PyResult};

use rosu_pp::model::mode::ConvertError as RsConvertError;

create_exception!(rosu_ppplus, ArgsError, PyException);
create_exception!(rosu_ppplus, ParseError, PyException);
create_exception!(rosu_ppplus, ConvertError, PyException);

/// Turns a `rosu_pp` conversion error into a Python `ConvertError`.
///
/// This cannot be a `From<RsConvertError> for PyErr` impl because both types
/// are foreign to this crate.
pub trait ConvertErrorExt<T> {
    fn py(self) -> PyResult<T>;
}

impl<T> ConvertErrorExt<T> for Result<T, RsConvertError> {
    fn py(self) -> PyResult<T> {
        self.map_err(|err| ConvertError::new_err(err.to_string()))
    }
}
