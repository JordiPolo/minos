use reqwest::StatusCode;

use checkers::test_status_code_equality;
use disparity::{Disparity, DisparityList, Location};
use schema;
use service::ServiceResponse;
use spec::Spec;

fn json_error() -> Disparity {
    Disparity::new(
        "Response could not be parsed as JSON. Responses must be proper JSON.",
        Location::empty(),
    )
}

// TODO: move somewhere else
fn is_application_defined_code(expected: StatusCode) -> bool {
    let appplication_codes = vec!["200", "400", "403", "404", "409", "422", "423"];
    appplication_codes
        .iter()
        .any(|&code| code == expected.as_str())
}
// pub fn is_middleware_code(&self) -> bool {
//     let appplication_codes = vec!["401", "500"];
//     appplication_codes.iter().find(|&&code| code == self.defined_code).is_some()
// }
// pub fn is_framework_code(&self) -> bool {
//     !(self.is_application_defined_code() || self.is_application_defined_code())
// }

// There are three outcomes here, the schema, a disparity or neither (ignore this case)
fn extract_schema(
    method: &openapi::v2::Operation,
    expected: StatusCode,
) -> Result<openapi::v2::Schema, Option<Disparity>> {
    let location = Location::new(vec![]);
    let defined_code = method.responses.get(expected.as_str());

    match defined_code {
        Some(defined) => match &defined.schema {
            Some(schema) => Ok(schema.clone()),
            None => {
                // TODO: Make this a configuraiton option? For now we don't complain if there are no schemas for error codes.
                // if is_application_defined_code(&expected) {
                //     Err(Some(Disparity::new(
                //             &format!("Expected that the endpoint to have a schema for {} but it is not there!", expected.as_str()),
                //             location.clone()))
                //         )
                // } else {
                Err(None)
                //  }
            }
        },
        None => {
            if is_application_defined_code(expected) {
                let error = Disparity::new(
                    &format!(
                        "The application responded with status code {} but this code is not documented in the openapi file!",
                        expected
                    ),
                    location.clone(),
                );
                Err(Some(error))
            } else {
                Err(None)
            }
        }
    }
}

pub fn check_and_validate(
    spec: &Spec,
    method: &openapi::v2::Operation,
    real_response: &ServiceResponse,
    expected: StatusCode,
) -> DisparityList {
    // First check if the returned status code is what we expect, if not we stop here.
    match test_status_code_equality(real_response.status, expected) {
        Some(disparity) => disparity.to_list(),
        None => {
            let v2_schema = extract_schema(method, expected);
            // Next check if the body contains JSON, if not, it may be ok if the file is ready for this
            // But it is an error if the file had a schema defining this code and there was no code
            match real_response.body {
                Err(_) => {
                    if v2_schema.is_ok() {
                        json_error().to_list()
                    } else {
                        DisparityList::new()
                    }
                }
                Ok(ref body) => {
                    // There are three outcomes here, the schema, a disparity or neither
                    match v2_schema {
                        Err(maybe_disparity) => match maybe_disparity {
                            None => DisparityList::new(),
                            Some(disparity) => disparity.to_list(),
                        },
                        Ok(s) => {
                            let schema = schema::Schema::new(spec.clone(), s.clone());
                            schema.validate(&body, &Location::empty())
                        }
                    }
                }
            }
        }
    }
}
