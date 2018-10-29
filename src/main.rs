extern crate json;
extern crate openapi;
extern crate rand;
extern crate regex;
extern crate reqwest;
extern crate serde_json;
extern crate structopt;
extern crate termcolor;
extern crate uuid;

mod checkers;
mod cli;
mod cli_args;
mod disparity;
mod mutation;
mod operation;
mod request_params;
mod service;
mod spec;
mod string_validator;

use checkers::check_status;
use disparity::{Disparity, DisparityList, Location};
use service::{Request, Service, ServiceResponse};
use spec::Spec;
use request_params::make_query_params;
use request_params::get_path_params;

fn json_error(location: &Location) -> Disparity {
    Disparity::new(
        "Response could not be parsed as JSON. Responses must be proper JSON.",
        location.clone(),
    )
}

// TODO: Experiment using yaml-rust instead of openapi crate to read the spec
mod schema;

//TODO: Divide this into several methods?
fn check_and_validate(
    spec: &Spec,
    method: &openapi::v2::Operation,
    real_response: &ServiceResponse,
    mutation: &mutation::Mutation,
    // code: &str,
    // status: reqwest::StatusCode,
    location: &mut Location,
) -> DisparityList {
    let mut result = DisparityList::new();
    let defined_code = method.responses.get(mutation.defined_code);

    let failed = result.option_push(check_status(
        real_response.status,
        mutation.expected,
        &location,
    ));
    if failed {
        return result;
    }

    match defined_code {
        Some(defined) => {
            match &defined.schema {
                Some(schema) => match real_response.value {
                    Ok(ref body) => {
                        let my_schema = schema::Schema::new(spec.clone(), schema.clone());
                        result.merge(my_schema.validate(&body, &location));
                    }
                    Err(_) => result.push(json_error(&location)),
                },
                None => {
                    let error = Disparity::new(
                        &format!("Expected that the endpoint to have a schema for {} but it is not there!", mutation.defined_code),
                        location.clone()
                    );
                    result.push(error);
                }
            }
        }
        None => {
            if mutation.is_application_defined_code() && (real_response.status == mutation.expected)
            {
                let error = Disparity::new(
                    &format!(
                        "The application responded with {} but the code is not documented in the openapi file!",
                        real_response.status
                    ),
                    location.clone(),
                );
                result.push(error);
            }
        }
    };

    result
}

//TODO: bring server back
fn main() {
    let config = cli_args::config();
    let spec = Spec::from_filename(&config.filename);
    // let mut result = DisparityList::new();
    let service = Service::new(&config, &spec.spec.base_path);

    for (path_name, methods) in spec.spec.paths.iter() {
        match operation::Operation::understand_operation(&spec, path_name, &methods) {
            // We don't understand what kind of operation is this
            None => continue,
            Some(operation) => {
                println!("\n \n **** Testing {} ****", path_name);

                for mutation in mutation::mutations_for_crud(operation.crud).iter() {
                    //TODO: check index mutations only
                    let mut location = Location::new(vec![
                        path_name,
                        &mutation.method,
                        &mutation.defined_code,
                        &mutation.content_type,
                    ]);

                    let request_parameters =
                        match make_query_params(&spec, &operation.method, &mutation.query_params) {
                            // No valid query params could be created to fulfill what this mutation
                            // wants to test so we just skip this one
                            None => continue,
                            Some(query_params) => query_params
                                .into_iter()
                                .map(|param| (param.name, param.value))
                                .collect(),
                        };

                    println!("{}", mutation.explanation);
                    // println!("{:?}", request_parameters);

                    let request = Request::new()
                        .path(path_name)
                        .query_params(request_parameters)
                        .content_type(mutation.content_type)
                        .set_method(mutation.method)
                        .path_params(get_path_params(&mutation.crud_operation));

                    let real_response = service.send(&request);

                    let disparities = check_and_validate(
                        &spec,
                        &operation.method,
                        &real_response,
                        mutation,
                        &mut location,
                    );
                    // result.merge(&disparities);  // TODO: add to list? Do we need this anymore?
                    if disparities.is_empty() {
                        cli::print_success("Test passed.")
                    } else {
                        cli::print_error(disparities);
                    }
                }
            }
        }
    }
    // println!("{}", result);
    //   */
}
