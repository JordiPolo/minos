//use json;
use disparity::*;
use openapi;
//use regex::*;

use json::JsonValue;
use reqwest::StatusCode;

pub fn check_status(
    response_status: StatusCode,
    desired_status: StatusCode,
    location: &Location,
) -> Option<Disparity> {
    if response_status == desired_status {
        None
    } else {
        let error = Disparity::new(
            &format!(
                "Response status inconsistency: Expected: {}, got {}. ",
                desired_status, response_status
            ),
            location.clone(),
        );
        Some(error)
    }
}

// Checks that the response has values of the same type as specified in the schema
pub fn check_response_type(
    value: &JsonValue,
    schema_type: &str,
    location: &Location,
) -> Option<Disparity> {
    let is_match = match *value {
        JsonValue::String(_) => schema_type == "string",
        // Short is a weird type made up but that library, it's really a &str or similar
        JsonValue::Short(_) => schema_type == "string",
        JsonValue::Number(_) => schema_type == "integer" || schema_type == "number",
        JsonValue::Boolean(_) => schema_type == "boolean",
        JsonValue::Array(_) => schema_type == "array",
        JsonValue::Object(_) => schema_type == "object",
        JsonValue::Null => true,
    };
    if !is_match {
        let error = Disparity::new(
            &format!(
                "Got {} from the service but described as {:?}",
                value, schema_type
            ),
            //TODO: improve location
            location.clone(),
        );
        Some(error)
    } else {
        None
    }
}

pub fn check_number_format(
    value: &JsonValue,
    _schema: &openapi::v2::Schema,
    _location: &Location,
) -> Option<Disparity> {
    match value {
        &JsonValue::Number(_) => None,
        _ => {
            panic!("We check the type before, this can't happen.");
            // Some(Disparity::new(
            //     &format!("The openAPI file says it is a number but we did not get a number."),
            //     //TODO: improve location
            //     location.clone(),
            // ))
        }
    }
}
