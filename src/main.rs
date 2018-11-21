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
mod mutation_instructions;
mod operation;
mod mutator;
mod schema;
mod service;
mod spec;
mod string_validator;
mod validator;
mod known_param;

use crate::service::Request;
use crate::service::Service;
use crate::spec::Spec;

// TODO: Zipkin
// TODO: Build test cases
// TODO: Combination of parameters

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
    let spec = Spec::from_filename(&config.filename).inline_everything();
    let service = Service::new(&config, &spec.base_path);
    let mutator = mutator::Mutator::new();

    //println!("{:?}", spec);

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
    let operations = spec.paths.iter().filter_map(|(path_name, methods)| {
        operation::Operation::create_supported_operation(path_name, &methods)
    });

    for operation in operations {
        let request_path = mutator.make_path(&operation.path_name);

        println!("\n \n **** Testing {} ****", operation.path_name);

        for mutation_instructions in mutation_instructions::instructions_for_operation(operation.crud).iter() {
            let request_parameters =
                match mutator.make_query_params(&operation.method, &mutation_instructions.query_params) {
                    // No valid query params could be created to fulfill what this mutation
                    // wants to test so we just skip this one
                    None => continue,
                    Some(query_params) => query_params
                        .into_iter()
                        .map(|param| (param.name, param.value))
                        .collect(),
                };

            cli::print_mutation_scenario(&request_path, mutation_instructions);

            let request = Request::new()
                .path(&request_path)
                .query_params(request_parameters)
                .content_type(mutation_instructions.content_type)
                .set_method(mutation_instructions.method);

            println!("Requesting {}", request);

            let real_response = service.send(&request);

            let disparities = validator::check_and_validate(
                &operation.method,
                &real_response,
                mutation_instructions.expected,
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
