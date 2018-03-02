#![feature(match_default_bindings)]

extern crate openapi;
extern crate regex;
extern crate serde_json;
extern crate reqwest;
extern crate json;
#[macro_use]
extern crate structopt;
extern crate termcolor;
//extern crate indicatif;

use regex::Regex;

use json::JsonValue;
use structopt::StructOpt;

use reqwest::StatusCode;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

//use indicatif::ProgressBar;

mod disparity;
use disparity::*;

mod checkers;
use checkers::*;

mod string_validator;
use string_validator::*;

mod service;
use service::*;

mod spec;
use spec::*;

mod cli_args;
use cli_args::*;


fn looks_like_index(path_name: &str) -> bool {
    let re = Regex::new(r"^/[\w|-]+$").unwrap();
    re.is_match(path_name)
}

fn json_ref_name(reference: &str) -> String {
    reference.split("/").last().unwrap().to_owned()
}


// TODO: convert this to return errors and return errors instead of the early crappy return of disparities
fn validate(
    spec: &Spec, //&std::collections::BTreeMap<std::string::String, openapi::v2::Schema>,
    schema: &openapi::v2::Schema,
    response: &JsonValue,
    location: &Location,
) -> DisparityList {
    let mut disparities = DisparityList::new();

    if schema.schema_type.is_none() {
        println!("We could not find a type at location {:?}. Types must always be specified.", location);
        return disparities;
    }

    let s_type = schema.schema_type.clone().unwrap();
   // println!("{:?} -> {:?}", location, s_type);

    // Incorrect type, fail here
    let type_disparity = check_response_type(response, &s_type, &location);
    if type_disparity.is_some() {
        disparities.option_push(type_disparity);
        return disparities;
    }

    // TODO: make an enum and a match instead of ifs
    if s_type == "array" {
        // TODO $ref is not done?
        // This is an empty array because we already checked the type before
        if response.is_empty() { return disparities; }
        let items = &schema.items.clone().unwrap();
        if items.ref_path.is_some() {
            let definition_name = items.ref_path.clone().unwrap();
            let definition = spec.resolve_definition(&definition_name);
            //let definition = definitions.get(&json_ref_name(&definition_name)).unwrap().clone();
            let new_location = location.clone().add(&json_ref_name(&definition_name));
            disparities.merge(validate(&spec, &definition, &response.members().as_slice()[0], &new_location));
        } else {
            // let schema_type = &items.schema_type;
        //    println!("TODO: Support arrays of strings, etc.");
        }
    } else if s_type == "object" {
        // Check that all the properties in the response are in the schema, and recurse on them
        let schema_properties = schema.properties.clone().unwrap().clone();
        for (property_name, property_value) in response.entries() {
            let property_schema = schema_properties.get(property_name);
            match property_schema {
                Some(new_schema) => {
                    disparities.merge(validate(&spec, new_schema, property_value, &location.add(property_name)));
                },
                None => {
                    let error = Disparity::new(
                        &format!("Got a response with a property {:?} not described in your openapi file", property_name),
                        //TODO: improve location and simplify message
                        location.clone(),
                    );
                    disparities.push(error);
                }
            }

        }
        // TODO: This works well, but do we really want to do it?
        // Check that the properties in the schema are there in the response. Don't need to recurse, done above.
        // for (schema_property_name, _schema_property_value) in schema_properties {
        //     if !response.has_key(&schema_property_name) {
        //             let error = Disparity::new(
        //                 &format!("The property {:?} described in your openapi file was not present in the real output.", schema_property_name),
        //                 //TODO: improve location and simplify message
        //                 location.clone(),
        //             );
        //             disparities.push(error);
        //     }
        // }
    } else if s_type == "string" {
        let validator = StringValidator::new(response, schema);
        disparities.option_push(validator.validate(&location));
    } else if s_type == "number" { //float and double
        disparities.option_push(check_number_format(response, schema, &location));
    } else {
        panic!("Unknown type {:?}", s_type);
    }
            //         JsonValue::Boolean(boolean) => {},
            //         JsonValue::Null => {},

    disparities
}




fn json_error(location: &Location) -> Disparity {
    Disparity::new("Response could not be parsed as JSON. Responses must be proper JSON.",location.clone())
}



// Road to v0.1:
// Dont blow up on booleans
// Try parameters, it should be ok
// Try malformed parameters, it should be 422 and json body
// Run and control of server

// Maybe:
// Experiment with parameters on paths for non index get routes

// TODO: Experiment using yaml-rust instead of openapi crate to read the spec


fn main() {
    // Set output text to yello
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow))).unwrap();

    let config = CLIArgs::from_args();

    let spec = Spec::from_filename(&config.filename);
 //   let definitions = spec::extract_definitions(&spec);
 //   let global_parameters = spec::extract_global_parameters(&spec);
    let mut result = DisparityList::new();
    let mut service = Service::new(&config, &spec.spec.base_path);

//    let bar = ProgressBar::new(spec.paths.len() as u64);

    for (path_name, methods) in spec.spec.paths.iter() {
//        bar.inc(1);
        // println!("{:?}", checked);
        // checked = checked + 1;
        // pb.set_position(checked);
        if looks_like_index(path_name) {
            // 200 + There must be always at least a success response in an index
            let defined_200 = spec::method_status_info(methods, "200").unwrap();
            let schema = spec::extract_schema(&defined_200).unwrap();
            let mut location = Location::new(vec![path_name, "get", "200"]);
            let real_response = service.call_success(path_name, None);

            match real_response.value {
                Ok(body) => {
                    if !result.option_push(check_status(real_response.status, StatusCode::Ok, &location)) {
                        result.merge(validate(&spec, &schema, &body, &mut location));
                    }
                },
                Err(_) => result.push(json_error(&location))
            }

            // 200 + Adding random parameters should break nothing
            let mut location = Location::new(vec![path_name, "get", "200", "adding_unknown_param"]);
            let real_response = service.call_success(path_name, Some(("trusmis", "mumi")));
            match real_response.value {
                Ok(body) => {
                    result.option_push(check_status(real_response.status, StatusCode::Ok, &location));
                    result.merge(validate(&spec, &schema, &body, &mut location));
                },
                Err(_) => result.push(json_error(&location))
            }



           // Can unwrap because we are in the index method
            // match &methods.clone().get.unwrap().parameters {
            //     Some(parameter) => {
            //         match &parameter[0] {
            //            openapi::v2::ParameterOrRef::Parameter{name, ..} => { println!("{:?}", name);},
            //            openapi::v2::ParameterOrRef::Ref{ref_path} => {println!("{:?}", ref_path);}
            //         }
            //     },
            //     None => {}
            // }


            // if methods.parameters.is_some() {
            //     methods.parameters.unwrap().iter().map(|param| {
            //         if let param = openapi::v2::ParameterOrRef::Ref {ref_path} {}
            //        //  match param {
            //        // //     p @ openapi::v2::ParameterOrRef::Parameter{..} => p,
            //        //      openapi::v2::ParameterOrRef::Ref{ref_path} => global_parameters.get(&json_ref_name(ref_path)).unwrap()
            //        //  }
            //     });
            // }



            //406
            let defined_406 = spec::method_status_info(methods, "406");
            let mut location = Location::new(vec![path_name, "get", "using_wrong_content_type"]);
            let real_response = service.call_failed_content_type(path_name);

            match defined_406 {
                // 406 defined in the spec, this should be used for a wrong content type
                Some(spec_response) => {
                    result.option_push(check_status(real_response.status, StatusCode::NotAcceptable, &location));
                    let error_schema = spec::extract_schema(&spec_response);
                    match error_schema {
                        Some(schema) => {
                            // Assume every schema is json : TODO: support non-json schemas
                            match real_response.value {
                                Ok(body) => result.merge(validate(&spec, &schema, &body, &mut location)),
                                Err(_) => result.push(json_error(&location)),
                            }
                        },
                        None => {}
                    }
                },
                // 406 not defined
                None => {
                    let error = if real_response.status == StatusCode::NotAcceptable {
                        Disparity::new(
                            &format!("Got 406 response for a wrong content type but it is not defined in the file."),
                            location.clone()
                        )
                    } else {
                        Disparity::new(
                            &format!("The status code when a wrong content type is provided is {} but should be 406.", real_response.status),
                            location.clone()
                        )
                    };
                    result.push(error);
                }
            }

            // 422

        }
    }
    println!("{}", result);
    service.kill();
}
