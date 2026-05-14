use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KernelError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("already exists: {0}")]
    AlreadyExists(String),
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("authorization failed: {0}")]
    AuthorizationFailed(String),
    #[error("certification failed: {0}")]
    CertificationFailed(String),
    #[error("type mismatch: {0}")]
    TypeMismatch(String),
    #[error("zero-vector violation: {0}")]
    ZeroVectorViolation(String),
    #[error("record integrity violation: {0}")]
    RecordIntegrityViolation(String),
    #[error("double reconstruction attempt: {0}")]
    DoubleReconstruction(String),
    #[error("contract error: {0}")]
    ContractError(String),
}

pub type Result<T> = core::result::Result<T, KernelError>;
