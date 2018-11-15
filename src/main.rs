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
mod schema;
mod mutation;
mod operation;
mod request_params;
mod service;
mod spec;
mod string_validator;
mod validator;

use spec::Spec;
use request_params::make_query_params;
use service::Service;
use service::Request;

// TODO consolidate known params in request_params
// TODO: Zipkin
// TODO: Build test cases
// TODO: Combination of parameters



// TODO: Remove after 1.31
// TODO edition 2018
#[global_allocator]
static A: std::alloc::System = std::alloc::System;


fn main() {
    let config = cli_args::config();
    let spec = Spec::from_filename(&config.filename);
    // let mut result = DisparityList::new();
    let service = Service::new(&config, &spec.spec.base_path);

    for (path_name, methods) in spec.spec.paths.iter() {
        match operation::Operation::create_operation(&spec, path_name, &methods) {
            // We don't understand what kind of operation is this
            None => continue,
            Some(operation) => {
                let request_path = request_params::make_path(path_name);

                println!("\n \n **** Testing {} ****", path_name);

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
                    // result.merge(&disparities);  // TODO: add to list? Do we need this anymore?
                    if disparities.is_empty() {
                        cli::print_success("Test passed.")
                    } else {
                        cli::print_error(disparities);
                    }
                    println!();
                }
            }
        }
    }
}
