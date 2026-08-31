use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum SottoError {
    #[error("{message}")]
    App {
        code: &'static str,
        message: String,
        recoverable: bool,
        action_hint: String,
    },
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl SottoError {
    pub fn app(
        code: &'static str,
        message: impl Into<String>,
        recoverable: bool,
        action_hint: impl Into<String>,
    ) -> Self {
        Self::App {
            code,
            message: message.into(),
            recoverable,
            action_hint: action_hint.into(),
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::App { code, .. } => code,
            Self::Sqlite(_) => "SQLITE",
            Self::Io(_) => "IO",
            Self::Json(_) => "JSON",
        }
    }
}

#[derive(Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
    pub action_hint: String,
}

impl From<SottoError> for ErrorBody {
    fn from(value: SottoError) -> Self {
        match value {
            SottoError::App {
                code,
                message,
                recoverable,
                action_hint,
            } => Self {
                code: code.to_string(),
                message,
                recoverable,
                action_hint,
            },
            other => Self {
                code: other.code().to_string(),
                message: other.to_string(),
                recoverable: false,
                action_hint: "See logs. Audio already captured is kept.".into(),
            },
        }
    }
}

pub type Result<T> = std::result::Result<T, SottoError>;
