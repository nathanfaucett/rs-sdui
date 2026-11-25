use a2ui::FunctionError;

pub const CANCELLED: &str = "CANCELLED";
pub const DISPOSED: &str = "DISPOSED";
pub const DUPLICATE: &str = "DUPLICATE";
pub const EXECUTION_ERROR: &str = "EXECUTION_ERROR";
pub const INVALID_CATALOG: &str = "INVALID_CATALOG";

pub const TIMEOUT: &str = "TIMEOUT";
pub const UNKNOWN_FUNCTION: &str = "UNKNOWN_FUNCTION";

pub fn error(code: &str, message: impl Into<String>) -> FunctionError {
    FunctionError {
        code: code.to_owned(),
        message: message.into(),
    }
}

pub fn invalid_call(message: impl Into<String>) -> FunctionError {
    error(FunctionError::INVALID_FUNCTION_CALL, message)
}

pub fn unknown_function(name: &str) -> FunctionError {
    error(
        UNKNOWN_FUNCTION,
        format!("Function '{name}' is not registered in the selected catalog"),
    )
}

pub fn execution_error(message: impl Into<String>) -> FunctionError {
    error(EXECUTION_ERROR, message)
}
