use thiserror::Error;

#[derive(Error, Debug)]
pub enum TableGenError {
    #[error("error creating struct: `{0}`")]
    CreateStruct(String),
    #[error("pointer is null")]
    NullPointer,
    #[error("unknown TableGen error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, TableGenError>;
