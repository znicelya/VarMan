#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_error_serializes_code_and_message() {
        let error = EnvError::permission_denied("需要管理员权限");
        let json = serde_json::to_value(&error).unwrap();
        assert_eq!(json["code"], "PERMISSION_DENIED");
        assert_eq!(json["message"], "需要管理员权限");
    }
}

use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EnvError {
    code: String,
    message: String,
}

impl EnvError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::new("PERMISSION_DENIED", message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message)
    }

    pub fn parse(message: impl Into<String>) -> Self {
        Self::new("PARSE_ERROR", message)
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new("IO_ERROR", message)
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new("UNSUPPORTED", message)
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}

impl fmt::Display for EnvError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl Error for EnvError {}

impl From<std::io::Error> for EnvError {
    fn from(error: std::io::Error) -> Self {
        Self::io(error.to_string())
    }
}
