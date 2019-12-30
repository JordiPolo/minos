use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DisparityError {
    #[error("Status code disparity. Expected {expected:?}, found {found:?}")]
    StatusDisparity {
        expected: StatusCode,
        found: StatusCode,
    },
    #[error("The application responded with status code {0} but this code is not documented in the openapi file!")]
    UndocumentedCode(StatusCode),

    #[error("The application responded with status code {0} but this code has no schema for it!")]
    SchemaNotFound(StatusCode),

    #[error("The application responded with malformed response. Can't parse as Json.")]
    JsonError,
    // #[error("the data for key `{0}` is not available")]
    // Redaction(String),
    // #[error("invalid header (expected {expected:?}, found {found:?})")]
    // InvalidHeader {
    //     expected: String,
    //     found: String,
    // },
    // #[error("unknown data store error")]
    // Unknown,
}

pub fn status_error(expected: StatusCode, found: StatusCode) -> DisparityError {
    DisparityError::StatusDisparity { expected, found }
}
