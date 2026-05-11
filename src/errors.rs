#[derive(Debug)]
pub struct Error {
    err: Box<ErrorCode>,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
enum ErrorCode {
    InvalidSignalType(String),
}

impl Error {
    pub(crate) fn syntax(msg: String) -> Self {
        Self {
            err: Box::new(ErrorCode::InvalidSignalType(msg)),
        }
    }
}
