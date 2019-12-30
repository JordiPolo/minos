use reqwest::StatusCode;
use openapi_utils::ResponseExt;

use crate::error;
use crate::service;

// pub fn is_middleware_code(&self) -> bool {
//     let appplication_codes = vec!["401", "500"];
//     appplication_codes.iter().find(|&&code| code == self.defined_code).is_some()
// }
// pub fn is_framework_code(&self) -> bool {
//     !(self.is_application_defined_code() || self.is_application_defined_code())
// }

fn is_application_defined_code(expected: StatusCode) -> bool {
    let appplication_codes = vec!["200", "400", "403", "404", "409", "422", "423"];
    appplication_codes
        .iter()
        .any(|&code| code == expected.as_str())
}

fn extract_schema<'a>(
    expected_status_code: StatusCode,
    expected_body: Option<&'a openapiv3::Response>
) -> Result<Option<&'a openapiv3::Schema>, error::DisparityError> {
    match expected_body { // Check if we even have the code specified in the contract
        None => {
            // If is something apps specify, it totally should be there and we error
            if is_application_defined_code(expected_status_code) {
                Err(error::DisparityError::UndocumentedCode(
                    expected_status_code,
                ))
            } else {
                Ok(None) // If not, it is ok, we just do not provide body
            }
        }
        Some(body) => {
            // We got the code, now check if the code has a schema define for it.
            match body.json_schema() {
                Some(body) => Ok(Some(body)),
                None => {
                    // TODO: Maybe we should not fail if for instance error codes do not have a schema
                    if is_application_defined_code(expected_status_code) {
                        Err(error::DisparityError::SchemaNotFound(
                            expected_status_code,
                        ))
                    } else {
                        Ok(None)
                    }
                }
            }
        }
    }
}

pub fn validate(
    response: &service::ServiceResponse,
    expected_status_code: StatusCode,
    expected_body: Option<&openapiv3::Response>
) -> Result<(), error::DisparityError> {
    if response.status != expected_status_code {
        return Err(error::status_error(
            expected_status_code,
            response.status,
        ));
    }

    let response_body = response.body.as_ref().map_err(|_| error::DisparityError::JsonError)?;

    let schema_body = extract_schema(expected_status_code, expected_body)?;
// This is to rewrite
//                             let schema = schema::Schema::new(s.clone());
//                             schema.validate(&body, &Location::empty())
    Ok(())
}
