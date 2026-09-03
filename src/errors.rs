// src/errors.rs
// use crate::der::Tag;
use thiserror::Error;
use std::num::TryFromIntError;

/// The main Error enum for our DER parser.
#[derive(Debug, Error)]
pub enum DerError {
    #[error("Unexpected end of data at position {pos}")]
    UnexpectedEndOfData { pos: usize },

    // #[error("Invalid Tag: expected {expected:#x}, found {found:#x}")]
    // InvalidTag { expected: u8, found: u8 },

    #[error("Invalid Length: expected {expected}, found {found}")]
    InvalidLength { expected: usize, found: usize },

    #[error("Data is not valid UTF-8")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("Integer conversion error")]
    IntConversion(#[from] TryFromIntError),

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Wrong Tag: expected {expected}, found {found}")]
    WrongTag { expected: String, found: String },

    // #[error("Unknown Tag: found {found:#x}")]
    // UnknownTag { found: u8 },

    // #[error("Custom Error: {0}")]
    // Custom(String),
}

/// A convenient type alias for Results
// pub type Result<T> = std::result::Result<T, DerError>;

#[derive(Debug, Error)]
pub enum ShoppingItemError {
    #[error("ShoppingItem DerError: {der_error}")]
    DerError { der_error: String },

    #[error("Input Error: {input_error}")]
    InputError { input_error: String },
}