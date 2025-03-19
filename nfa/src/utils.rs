use std::fmt;
use std::io;

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    ParseInt(std::num::ParseIntError),
    FileFormat(String), // For general file format problems
    MissingEnd,
}

// Implement From traits for easy conversion
impl From<io::Error> for LoadError {
    fn from(err: io::Error) -> LoadError {
        LoadError::Io(err)
    }
}

impl From<std::num::ParseIntError> for LoadError {
    fn from(err: std::num::ParseIntError) -> LoadError {
        LoadError::ParseInt(err)
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoadError::Io(err) => write!(f, "IO error: {}", err),
            LoadError::ParseInt(err) => write!(f, "Parse error: {}", err),
            LoadError::FileFormat(msg) => write!(f, "File format error: {}", msg), // Use the String
            LoadError::MissingEnd => write!(f, "File format error: Missing 'END'"),
        }
    }
}
