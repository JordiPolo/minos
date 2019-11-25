mod checkers;
mod cli;
mod cli_args;
mod disparity;
mod mutation_instructions;
mod operation;
mod mutator;
//mod schema;
mod service;
mod spec;
//mod string_validator;
//mod validator;
mod known_param;

use crate::service::Request;
use crate::service::Service;
use openapi_utils::ReferenceOrExt;
use openapi_utils::ServerExt;
use openapi_utils::SpecExt;


struct Scenario<'a> {
    endpoint: operation::Endpoint,
    instructions: mutation_instructions::MutationInstruction<'a>,
}

impl<'a> Scenario<'a> {
    fn new(endpoint: operation::Endpoint, instructions: mutation_instructions::MutationInstruction<'a>) -> Self {
        Scenario {
            endpoint,
            instructions,
        }
    }
}

fn main() {
    let config = cli_args::config();
    println!("{:?}", config);
    let spec = spec::read(&config.filename).deref_all();
    let service = Service::new(&config, spec.servers[0].base_path());

    let mutator = mutator::Mutator::new(&spec);


    // Create endpoints from the spec file. Filter out the ones we don't understand
    // Eventually this will be all the stuff in the file.
    let endpoints = spec.paths.iter().filter_map(|(path_name, methods)| {
        operation::Endpoint::create_supported_endpoint(path_name, methods.to_item_ref())
    });

    let scenarios = endpoints.fold(Vec::new(), |mut acc, endpoint| {
        let instructions = mutation_instructions::instructions_for_operation(endpoint.crud.clone());
        for instruction in instructions {
            acc.push(Scenario::new(endpoint.clone(), instruction));
        }
        acc
    });


    let requests = scenarios.into_iter().map(|scenario| {
        let request_path = mutator.make_path(&scenario.endpoint.path_name);
        let request_parameters =
        match mutator.make_query_params(&scenario.endpoint.method, &scenario.instructions.query_params) {
            // No valid query params could be created to fulfill what this mutation
            // wants to test so we just skip this one
            None => panic!(),
            Some(query_params) => query_params
                .into_iter()
                .map(|param| (param.name, param.value))
                .collect(),
        };

        cli::print_mutation_scenario(&request_path, &scenario.instructions);

        Request::new()
            .path(request_path)
            .query_params(request_parameters)
            .content_type(scenario.instructions.content_type)
            .set_method(scenario.instructions.method)
    });

    // Run each scenario, get the response and valiate it with what we expect
    for request in requests {
        println!("Requesting {}", request);

        let real_response = service.send(&request);

        // let disparities = validator::check_and_validate(
        //     &operation.method,
        //     &real_response,
        //     mutation_instructions.expected,
        // );
        let disparities = disparity::DisparityList::new();
        if disparities.is_empty() {
            cli::print_success("Test passed.")
        } else {
            cli::print_error(disparities);
        }
        println!();
    }

}
