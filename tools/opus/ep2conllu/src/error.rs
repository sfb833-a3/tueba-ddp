use thiserror::Error;
use xml::reader;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error(transparent)]
    Xml(#[from] reader::Error),

    #[error("empty token")]
    EmptyTokenError,
}
