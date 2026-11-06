use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
}
impl AppError {
    pub fn invalid(code: &str) -> Self {
        Self {
            code: code.into(),
            message: code.into(),
        }
    }
}
impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        Self {
            code: "database_error".into(),
            message: error.to_string(),
        }
    }
}
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self {
            code: "file_error".into(),
            message: error.to_string(),
        }
    }
}
impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self {
            code: "invalid_json".into(),
            message: error.to_string(),
        }
    }
}
pub type AppResult<T> = Result<T, AppError>;
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for AppError {}
