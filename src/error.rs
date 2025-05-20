use std::fmt;

#[derive(Debug)]
pub enum OptzError {
  MissingArgument,
  Parse(String),
}

impl std::fmt::Display for OptzError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      OptzError::MissingArgument => write!(f, "Missing argument"),
      OptzError::Parse(msg) => write!(f, "{}", msg),
    }
  }
}

impl std::error::Error for OptzError {}

pub type Result<T> = std::result::Result<T, OptzError>;
