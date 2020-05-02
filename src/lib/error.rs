#[derive(Debug)]
pub enum Error {
    LogOpen(String),
    KeyNotFound,
    LogReaderNotFound,
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Error::LogOpen("error while trying to open a log".to_owned())
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_: std::str::Utf8Error) -> Self {
        Error::LogOpen("error while trying to deserialize a log".to_owned())
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(_: serde_json::error::Error) -> Self {
        Error::LogOpen("error while trying to deserialize a log".to_owned())
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(_: std::num::ParseIntError) -> Self {
        Error::LogOpen("error in probably while fetching number from log name".to_owned())
    }
}
