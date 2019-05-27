use std::fmt;

pub use failure::Error;

#[derive(Debug)]
struct MessageError {
  message: String
}

impl fmt::Display for MessageError {
  fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "{}", self.message)
  }
}

impl std::error::Error for MessageError {
  fn description(&self) -> &str {
    &self.message
  }
}

pub type Try<Value = ()> = Result<Value, Error>;

pub fn error(message: String) -> Error {
    Error::from(MessageError{message})
}
