use serde::Serialize;
use whitebase_interface::InterfaceError;

#[derive(Debug, Serialize)]
pub(crate) struct CommandError {
    code: &'static str,
    message: String,
}

impl CommandError {
    pub(crate) fn internal(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl From<InterfaceError> for CommandError {
    fn from(error: InterfaceError) -> Self {
        Self {
            code: error.code(),
            message: error.to_string(),
        }
    }
}
