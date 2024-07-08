#[derive(Debug)]
pub enum LoadHandlerError {
    RegexError(regex::Error)
}

impl From<regex::Error> for LoadHandlerError {
    fn from(value: regex::Error) -> Self {
        LoadHandlerError::RegexError(value)
    }
}