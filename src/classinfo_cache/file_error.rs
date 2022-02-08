use std::fmt;

pub enum FileError {
    FileSystem(std::io::Error),
    Parse(serde_json::Error),
    JoinError,
    PathError,
}

impl From<serde_json::Error> for FileError {
    fn from(error: serde_json::Error) -> FileError {
        FileError::Parse(error)
    }
}

impl From<std::io::Error> for FileError {
    fn from(error: std::io::Error) -> FileError {
        FileError::FileSystem(error)
    }
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileError::FileSystem(s) => write!(f, "{}", s),
            FileError::Parse(s) => write!(f, "{}", s),
            FileError::PathError => write!(f, "Path conversion to string failed"),
            FileError::JoinError => write!(f, "Join error"),
        }
    }
}