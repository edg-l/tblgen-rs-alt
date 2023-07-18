use thiserror::Error;

#[derive(Error, Debug)]
pub enum TableGenError {
    #[error("error creating struct: `{0}`")]
    CreateStruct(String),
    #[error("pointer is null")]
    NullPointer,
    #[error("invalid bit range")]
    InvalidBitRange,
    #[error("interior null byte in string")]
    StringNulError(#[from] std::ffi::NulError),
    #[error("unknown TableGen error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, TableGenError>;
