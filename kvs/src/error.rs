#[derive(Debug)]
pub enum Error {
    LogOpen(String),
    KeyNotFound,
    LogReaderNotFound,
    InsertError,
    RemoveError,
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

impl From<std::net::AddrParseError> for Error {
    fn from(_: std::net::AddrParseError) -> Self {
        Error::LogOpen("error during parsing address".to_owned())
    }
}

impl From<sled::Error> for Error {
    fn from(_: sled::Error) -> Self {
        Error::LogOpen("error from SLED side".to_owned())
    }
}
