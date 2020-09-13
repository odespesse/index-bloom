use std::error::Error as StdError;
use std::fmt;
use std::num::ParseIntError;

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    HashWord(ParseIntError),
}

impl StdError for Error {
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
          Error::HashWord(error) => write!(f, "Error while hashing word : {}", error),
        }
    }
}

