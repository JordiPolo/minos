//#![feature(match_default_bindings)]
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

use reqwest::StatusCode;

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
mod cli;

mod mutation;

mod request_params;
//use request_params::*;
//use std::collections::BTreeMap;

fn has_url_params(path_name: &str) -> bool {
    let re = Regex::new(r"^/[\w|-]+$").unwrap();
    if re.is_match(path_name)
    {
        false
    } else {
        true
    } 
}
//GET, no path params, no required query params
fn looks_like_index(spec: &Spec, path_name: &str, methods: &openapi::v2::PathItem) -> bool {
    if has_url_params(path_name) {
        return false;
    }

    match &methods.get {
        Some(get) => {
            // Ok, so far so good. 
            // Now we will not allow any required parameter. Maybe in the future we can improve on this
            let params_option = &get.parameters;
            match params_option {
                Some(params) => {
                    params.iter().all(| ref p| {
                        match resolve_parameter_ref(spec, p).required {
                            Some(required) => !required,
                            None => true,
                        }
                    })
                },
                None => true
            }          
        },
        None => false
    }
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

    //TODO: match the option.
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


// 0.2:
// Experiment with parameters on paths for non index get routes

// TODO: Experiment using yaml-rust instead of openapi crate to read the spec

fn resolve_parameter_ref( spec: &Spec, param_or_ref: &openapi::v2::ParameterOrRef) -> openapi::v2::Parameter {
    match param_or_ref.clone() {
        openapi::v2::ParameterOrRef::Parameter{name, location, required, schema, unique_items, param_type, format, description, minimum, maximum, default, enum_values} => {
            openapi::v2::Parameter {name, location, required, schema, unique_items, param_type, format, description, minimum, maximum, default, enum_values}
        },
        openapi::v2::ParameterOrRef::Ref{ref_path} => spec.resolve_parameter(&ref_path)
    }
}

// fn check_200() {
//     let real_response = service.call_success(path_name, None);

//     match real_response.value {
//         Ok(body) => {
//             if !result.option_push(check_status(real_response.status, StatusCode::Ok, &location)) {
//                 result.merge(validate(&spec, &schema, &body, &mut location));
//             }
//         },
//         Err(_) => result.push(json_error(&location))
//     }

// }

//TODO: Divide this into several methods?
fn check_and_validate(
    spec: &Spec, //&std::collections::BTreeMap<std::string::String, openapi::v2::Schema>,
    methods: &openapi::v2::PathItem,
    code: &str,
    real_response: &ServiceResponse,
    status: reqwest::StatusCode,
    mut location: &mut Location,
) -> DisparityList {
    let mut result = DisparityList::new();
    let defined_code = spec::method_status_info(methods, code);

    match defined_code {
        Some(defined) => {
            let failed = result.option_push(check_status(real_response.status, status, &location));
            match spec::extract_schema(&defined) {
                Some(schema) => {
                    if !failed {
                        match real_response.value {
                            Ok(ref body) => {
                                    result.merge(validate(&spec, &schema, &body, &mut location));
                            },
                            Err(_) => result.push(json_error(&location))
                        }
                    }
                },
                None => {
                    let error = Disparity::new(
                        &format!("Expected that the endpoint to have a schema for {} but it is not there!", code),
                        location.clone()
                    );
                    result.push(error);
                }
            }
        },
        None => {
            let error = Disparity::new(
                &format!("Expected that the endpoint define {} but it is not there!", code),
                location.clone()
            );
            result.push(error);
        }
    }

    result
}






fn main() {
    let config = cli_args::config();

    let spec = Spec::from_filename(&config.filename);
 //   let definitions = spec::extract_definitions(&spec);
 //   let global_parameters = spec::extract_global_parameters(&spec);
    let mut result = DisparityList::new();
    let service = Service::new(&config, &spec.spec.base_path);

//    let bar = ProgressBar::new(spec.paths.len() as u64);

  //  let paths = spec.spec.paths.clone();
    // let looks_like_create = paths.iter().filter(|&(path, methods)|
    //     methods.post.is_some() &&
    //     path.chars().last().is_some() &&
    //     path.chars().last().unwrap() !='}'
    // );

    // let mut operations: BTreeMap<String, (String, openapi::v2::PathItem)> = BTreeMap::new();
    // for (path_name, methods) in paths.clone() {
    //     if methods.post.is_some() && methods.clone().post.unwrap().operation_id.is_some() {
    //         let operation_id = methods.clone().post.unwrap().operation_id.unwrap();
    //         let resource = operation_id.split(".").nth(0).unwrap().to_string();
    //         operations.insert(resource, (path_name.clone(), methods.clone()));
    //     }
    //     if methods.get.is_some() && methods.clone().get.unwrap().operation_id.is_some() {
    //         let operation_id = methods.clone().get.unwrap().operation_id.unwrap();
    //         let resource = operation_id.split(".").nth(0).unwrap().to_string();
    //         operations.insert(resource, (path_name.clone(), methods.clone()));
    //     }
    //     if methods.delete.is_some() && methods.clone().delete.unwrap().operation_id.is_some() {
    //         let operation_id = methods.clone().delete.unwrap().operation_id.unwrap();
    //         let resource = operation_id.split(".").nth(0).unwrap().to_string();
    //         operations.insert(resource, (path_name.clone(), methods.clone()));
    //     }
    // }

    // for (resource, &(path_name, methods)) in operations.iter() {
    //     if methods.post.is_some() {

    //     }
    // }



// /*



//     impl Mutation {
//         fn success_no_query(path_name: &str) {
//             // 200 + There must be always at least a success response in an index
//             let mut location = Location::new(vec![path_name, "get", "200"]);
//             println!("Calling {} with no parameters. Expecting 200.", &path_name);
//             let real_response = service.call_success(path_name, None);
// //            result.merge(check_and_validate(&spec, &methods, "200", &real_response, StatusCode::OK, &mut location));
//             let new_result = check_and_validate(&spec, &methods, "200", &real_response, StatusCode::OK, &mut location);            
//         }
//     }


    for (path_name, methods) in spec.spec.paths.iter() {
//        bar.inc(1);
        // println!("{:?}", checked);
        // checked = checked + 1;
        // pb.set_position(checked);
        if looks_like_index(&spec, path_name, &methods) {
            println!("\n \n TEST ****  {:?} ****", path_name);

            //TODO: Only get the mutations for index.
            for mutation in mutation::mutations().iter() {
                println!("In testing area");
                //TODO: check index mutations only
                let mut location = Location::new(vec![path_name, &mutation.method, &mutation.defined_code]);
                let real_response = service.call_success(path_name, mutation::make_query_params(mutation.query_params));
                let new_result = check_and_validate(&spec, &methods, mutation.defined_code, &real_response, mutation.expected, &mut location);
                cli::print_success(new_result);
            }

//             // 200 + There must be always at least a success response in an index
//             let mut location = Location::new(vec![path_name, "get", "200"]);
//             println!("Calling {} with no parameters. Expecting 200.", &path_name);
//             let real_response = service.call_success(path_name, None);
// //            result.merge(check_and_validate(&spec, &methods, "200", &real_response, StatusCode::OK, &mut location));
//             let new_result = check_and_validate(&spec, &methods, "200", &real_response, StatusCode::OK, &mut location);
//             // match new_result {
//             //     None => print_color("Test passed.", Color::Green),
//             //     Some(failure) => print_color(failure, Color::Green),
//             // }
//             cli::print_error("test");

//             cli::print_success(new_result);
//            // println!("{}", new_result);

//             // 200 + Adding random parameters should break nothing
//             let mut location = Location::new(vec![path_name, "get", "200", "adding_unknown_param"]);
//             println!("Calling {} with extra unknown parameters. Expecting 200.", &path_name);
//             let real_response = service.call_success(path_name, Some(("trusmis", "mumi")));
//            // result.merge(check_and_validate(&spec, &methods, "200", &real_response, StatusCode::OK, &mut location));
//             let new_result = check_and_validate(&spec, &methods, "200", &real_response, StatusCode::OK, &mut location);
//             cli::print_error(new_result);

            // 200 + allowed parameter with proper value should break nothing
            match request_params::get_proper_param(&spec, &methods) {
                Some(param) => {
                    let mut location = Location::new(vec![path_name, "get", "200", "adding proper param", &param.name]);
                    let real_response = service.call_success(path_name, Some((&param.name, &param.value)));
                    result.merge(check_and_validate(&spec, &methods, "200", &real_response, StatusCode::OK, &mut location));
                },
                None => {},
            }

            // 200 + allowed parameter with a wrong value should give a 422
            match request_params::get_proper_param(&spec, &methods) {
                Some(param) => {
                    let mut location = Location::new(vec![path_name, "get", "422", "adding improper param", &param.name]);
                    let real_response = service.call_success(path_name, Some((&param.name, "nooooo")));
                    result.merge(check_and_validate(&spec, &methods, "422", &real_response, StatusCode::UNPROCESSABLE_ENTITY, &mut location));
                },
                None => {},
            }


         //   println!("{:?}", params);
          //  let params = get_method.parameters.map(|param| param.into_iter().map(|p| resolve_parameter_ref(&spec, &p)));

            // match params {
            //     Some(parameters) => {}
            //     None => _
            // }

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

            // 405
            // Rails can't do this
            // let mut location = Location::new(vec![path_name, "get", "200", "using_disallowed_method"]);
            // let method_name = spec::get_random_undefined_method(methods);
            // let real_response = service.call_with_method(path_name, &method_name);

            // let failed_405 = check_status(real_response.status, StatusCode::MethodNotAllowed, &location);
            // result.option_push(failed_405);


            //406
            // let mut location = Location::new(vec![path_name, "get", "using_wrong_content_type"]);
            // let real_response = service.call_failed_content_type(path_name);
            // result.merge(check_and_validate(&spec, &methods, "406", &real_response, StatusCode::NOT_ACCEPTABLE, &mut location));


        }
    }
   // println!("{}", result);
 //   */
    // service.kill();

}
