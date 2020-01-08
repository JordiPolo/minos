use reqwest::StatusCode;
use std::fmt;
use thiserror::Error;

#[derive(Debug)]
pub struct PropertyError {
    path: String,
    category: String,
    details: String,
}

#[derive(Debug)]
pub struct ObjectError(Vec<PropertyError>);

impl fmt::Display for ObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in self.0.iter() {
            writeln!(
                f,
                "{}:\n\t{}. {}.",
                error.path, error.category, error.details
            )?;
        }
        write!(f, "") // Hack to avoid complains
    }
}

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

    #[error("The application data but there is no OpenAPI schema to match it. Data {0}")]
    SchemaNotFoundForData(serde_json::Value),

    #[error("The body of the response is incorrect.\n{0}")]
    BodySchemaIncorrect(ObjectError),
}

pub fn status_error(expected: StatusCode, found: StatusCode) -> DisparityError {
    DisparityError::StatusDisparity { expected, found }
}

pub fn body_schema_incorrect(errors: valico::ValicoErrors) -> DisparityError {
    DisparityError::BodySchemaIncorrect(ObjectError(
        errors
            .iter()
            .map(|error| PropertyError {
                path: str::replace(error.get_path(), "/0", "Body Root"),
                category: error.get_title().to_string(),
                details: error.get_detail().unwrap_or("").to_string(),
            })
            .collect(),
    ))
}
