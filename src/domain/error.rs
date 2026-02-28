use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum DomainError {
    Validation(String),
    NotFound(String),
    AlreadyExists(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::Validation(msg) => write!(f, "入力エラー: {}", msg),
            DomainError::NotFound(msg) => write!(f, "見つかりません: {}", msg),
            DomainError::AlreadyExists(msg) => write!(f, "既に存在します: {}", msg),
        }
    }
}

// 文字列への変換（UI表示用）
impl From<DomainError> for String {
    fn from(err: DomainError) -> Self {
        err.to_string()
    }
}
