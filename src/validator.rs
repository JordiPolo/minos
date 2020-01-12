use http::StatusCode;
use openapi_utils::ResponseExt;
//use openapi_utils::SchemaExt;

use crate::error;
use crate::scenario::ScenarioExpectation;
use crate::service;

pub fn validate(
    response: service::ServiceResponse,
    expectation: ScenarioExpectation,
) -> Result<(), error::DisparityError> {
    if response.status != expectation.status_code {
        return Err(error::status_error(
            expectation.status_code,
            response.status,
        ));
    }

    match response.content_type {
        None => {
            return Err(error::DisparityError::ContentTypeIncorrect(String::from(
                "empty content type",
            )))
        }
        Some(content_type) => {
            // Some servers respond adding the charset to application json which is incorrect
            // but let's be lenient for now
            if !(content_type.contains(&expectation.content_type)) {
                return Err(error::DisparityError::ContentTypeIncorrect(content_type));
            }
        }
    }

    let response_body = response
        .body
        .map_err(|_| error::DisparityError::JsonError)?;

    let schema_body = extract_schema(expectation.status_code, expectation.body)?;

    validate_schema(&response_body, schema_body)
}

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
    expected_body: Option<&'a openapiv3::Response>,
) -> Result<Option<&'a openapiv3::Schema>, error::DisparityError> {
    match expected_body {
        // Check if we even have the code specified in the contract
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
                        Err(error::DisparityError::SchemaNotFound(expected_status_code))
                    } else {
                        Ok(None)
                    }
                }
            }
        }
    }
}

fn validate_schema(
    body: &serde_json::Value,
    schema: std::option::Option<&openapiv3::Schema>,
) -> Result<(), error::DisparityError> {
    // Is ok the schema to be empty if there is data to check anyways
    // But if there is anything there we should be having a schema to match against!
    if schema.is_none() {
        if body == &serde_json::Value::Null {
            return Ok(());
        } else {
            return Err(error::DisparityError::SchemaNotFoundForData(body.clone()));
        }
    }

    // TODO: Also would be good to check the string format. Reuse the StringValidator

    let json_v4_schema = openapi_schema_to_json_schema(schema.unwrap());

    let mut scope = valico::json_schema::Scope::new();
    let valico = scope
        .compile_and_return(json_v4_schema, false)
        .expect("Improper creation of schema");
    let mut state = valico.validate(&body);

    if state.is_valid() {
        Ok(())
    } else {
        // println!("Response body:\n {}", body);
        return Err(error::body_schema_incorrect(&mut state.errors));
    }
}

// TODO: Do all the proper conversions here with all the differences between formats.
// Most notably   nullable: true  -> [type, null]
// See https://github.com/mikunn/openapi-schema-to-json-schema
fn openapi_schema_to_json_schema(schema_data: &openapiv3::Schema) -> serde_json::Value {
    let serialized = serde_json::to_string(&schema_data).expect("Improper serialization of schema");

    let json_v4_schema: serde_json::Value =
        serde_json::from_str(&serialized).expect("Improper deser of schema");
    json_v4_schema
}
