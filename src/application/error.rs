use std::fmt;

use crate::domain::DomainError;

#[derive(Debug, Clone, PartialEq)]
pub enum AppError {
    Input(String),
    Domain(String),
    Persistence(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Input(msg) => write!(f, "入力エラー: {}", msg),
            AppError::Domain(msg) => write!(f, "処理エラー: {}", msg),
            AppError::Persistence(msg) => write!(f, "永続化エラー: {}", msg),
        }
    }
}

impl From<DomainError> for AppError {
    fn from(value: DomainError) -> Self {
        Self::Domain(value.to_string())
    }
}
