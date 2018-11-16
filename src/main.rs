extern crate chrono;
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
mod schema;
mod service;
mod spec;
mod string_validator;
mod validator;

use request_params::make_query_params;
use service::Request;
use service::Service;
use spec::Spec;

// TODO consolidate known params in request_params
// TODO: Zipkin
// TODO: Build test cases
// TODO: Combination of parameters

// TODO: Remove after 1.31
// TODO edition 2018
#[global_allocator]
static A: std::alloc::System = std::alloc::System;

// struct Scenario<'a> {
//     request: service::Request<'a>,
//     expected: reqwest::StatusCode,
// }

// struct EndpointUnderTest<'a> {
//     path_name: String,
//     operation: operation::Operation,
//     scenarios : Vec<Scenario<'a>>,
// }

// impl<'a> EndpointUnderTest<'a> {
//     fn new(path_name: &String, operation: operation::Operation, scenarios: Vec<Scenario<'a>>) -> Self {
//         EndpointUnderTest {
//             path_name: path_name.to_owned(),
//             operation,
//             scenarios
//         }
//     }
// }

// fn request_for_operation<'a>(spec: &spec::Spec, operation: &'a operation::Operation, mutation: &'a mutation::Mutation) -> Option<service::Request<'a>> {
//     let request_path = request_params::make_path(&operation.path_name);
//     let request_parameters: Option<Vec<(String, String)>> =
//     make_query_params(&spec, &operation.method, &mutation.query_params).map(|query_params| {
//         query_params.into_iter().map(|param| (param.name, param.value)).collect()
//     });

//     if request_parameters.is_none() { return None };

//     let request = Request::new()
//         .path(&request_path)
//         .query_params(request_parameters.unwrap())
//         .content_type(mutation.content_type)
//         .set_method(mutation.method);

//     Some(request)
// }

fn main() {
    let config = cli_args::config();
    let spec = Spec::from_filename(&config.filename);
    let service = Service::new(&config, &spec.spec.base_path);

    // tests.iter().map(|test| {
    //     let request_path = request_params::make_path(test.path_name);
    //     let scenarios = mutation::mutations_for_crud(operation.crud).iter().map(|mutation| {
    //         let request_parameters =
    //             match make_query_params(&spec, &operation.method, &mutation.query_params) {
    //                 // No valid query params could be created to fulfill what this mutation
    //                 // wants to test so we just skip this one
    //                 None => continue,
    //                 Some(query_params) => query_params
    //                     .into_iter()
    //                     .map(|param| (param.name, param.value))
    //                     .collect(),
    //             };

    //         let request = Request::new()
    //             .path(&request_path)
    //             .query_params(request_parameters)
    //             .content_type(mutation.content_type)
    //             .set_method(mutation.method);
    //             //.path_params(get_path_params(&mutation.crud_operation));
    //         });
    //         Scenario {
    //             request,
    //             expected: mutation.expected,
    //         });
    //     test.scenarios = scenarios;
    // })

    // Create operations from the spec file. Filter out the ones we don't understand
    let operations = spec.spec.paths.iter().filter_map(|(path_name, methods)| {
        operation::Operation::create_operation(path_name, &methods)
    });

    for operation in operations {
        let request_path = request_params::make_path(&operation.path_name);

        println!("\n \n **** Testing {} ****", operation.path_name);

        for mutation in mutation::mutations_for_crud(operation.crud).iter() {
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

            cli::print_mutation_scenario(&request_path, mutation);

            let request = Request::new()
                .path(&request_path)
                .query_params(request_parameters)
                .content_type(mutation.content_type)
                .set_method(mutation.method);
            //.path_params(get_path_params(&mutation.crud_operation));

            println!("Requesting {}", request);

            let real_response = service.send(&request);

            let disparities = validator::check_and_validate(
                &spec,
                &operation.method,
                &real_response,
                mutation.expected,
            );
            if disparities.is_empty() {
                cli::print_success("Test passed.")
            } else {
                cli::print_error(disparities);
            }
            println!();
        }
    }
}
