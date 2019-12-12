//mod checkers;
mod cli;
mod cli_args;
//mod disparity;
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


struct ScenarioExecution<'a> {
    scenario: Scenario<'a>,
    request: Request<'a>,
}

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

    let o_endpoints: Vec<Option<operation::Endpoint>>;

    // Create endpoints from the spec file. Filter out the ones we don't understand
    // Eventually this will be all the stuff in the file.
    o_endpoints = spec.paths.iter().flat_map(|(path_name, methods)| {
        operation::Endpoint::create_supported_endpoint(path_name, methods.to_item_ref())
    }).collect();

    // TODO This is only done to rilter out None , there needs to be a better way
    let endpoints = o_endpoints.into_iter().filter_map(|e| e);


    let scenarios = endpoints.fold(Vec::new(), |mut acc, endpoint| {
        let instructions = mutation_instructions::instructions_for_operation(endpoint.crud.clone());
        for instruction in instructions {
            acc.push(Scenario::new(endpoint.clone(), instruction));
        }
        acc
    });


    let scenario_executions = scenarios.into_iter().map(|scenario| {
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

        let request = Request::new()
            .path(request_path)
            .query_params(request_parameters)
            .content_type(scenario.instructions.content_type)
            .set_method(scenario.instructions.method);
        ScenarioExecution { scenario, request }
    });

    // Run each scenario execution, get the response and validate it with what we expect
    for execution in scenario_executions {
        //println!("Requesting {:?}", request);

        let real_response = service.send(&execution.request);

        if real_response.status != execution.scenario.instructions.expected {
            println!("Got {}", real_response.status);
            cli::print_error("Test failed.")
        } else {
            cli::print_success("Test passed.")
        }

        // let disparities = validator::check_and_validate(
        //     &operation.method,
        //     &real_response,
        //     mutation_instructions.expected,
        // );
        // let disparities = disparity::DisparityList::new();
        // if disparities.is_empty() {
        //     cli::print_success("Test passed.")
        // } else {
        //     cli::print_error(disparities);
        // }
        println!();
    }

}
