use std::error::Error;
use std::fmt;
use std::io;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum InputError {
    IO(io::Error),
    Parse(ParseIntError),
    RegexError(String),
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InputError::IO(ref err) => err.fmt(f),
            InputError::Parse(ref err) => err.fmt(f),
            InputError::RegexError(ref input) => {
                write!(f, "Could not match input {} to regex", input)
            }
        }
    }
}

impl Error for InputError {}

impl From<ParseIntError> for InputError {
    fn from(err: ParseIntError) -> InputError {
        InputError::Parse(err)
    }
}

impl From<io::Error> for InputError {
    fn from(err: io::Error) -> InputError {
        InputError::IO(err)
    }
}
