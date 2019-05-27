mod cert;

pub use cert::*;
pub use failure::Error;
use std::io;

pub fn error(err: String) -> Error {
    Error::from(io::Error::new(io::ErrorKind::Other, err))
}

pub type Try<Value = ()> = Result<Value, Error>;
