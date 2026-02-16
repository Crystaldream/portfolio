use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(Box<surrealdb::Error>),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("External service error: {0}")]
    External(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl From<surrealdb::Error> for AppError {
    fn from(err: surrealdb::Error) -> Self {
        AppError::Database(Box::new(err))
    }
}

impl AppError {
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound(resource.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn external(msg: impl Into<String>) -> Self {
        Self::External(msg.into())
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::External(_) => StatusCode::BAD_GATEWAY,
            Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.to_string();
        tracing::error!(error = %message, status = %status, "Request failed");
        (status, message).into_response()
    }
}
