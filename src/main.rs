//mod checkers;
mod cli;
mod cli_args;
mod error;
mod mutation_instructions;
mod mutator;
mod operation;
//mod schema;
mod service;
mod spec;
//mod string_validator;
mod validator;
mod known_param;

use crate::service::{Request, Service};
use openapi_utils::{OperationExt, ReferenceOrExt, ServerExt, SpecExt};
use reqwest::StatusCode;

struct ScenarioExecution<'a> {
    scenario: Scenario<'a>,
    request: Request<'a>,
}

impl<'a> ScenarioExecution<'a> {
    // pub fn method(&self) -> openapiv3::Operation {
    //     self.scenario.endpoint.method.clone()
    // }

    pub fn expected_status_code(&self) -> StatusCode {
        self.scenario.instructions.expected
    }
    pub fn expected_body(&self) -> Option<&openapiv3::Response> {
        self.scenario
            .endpoint
            .method
            .response(self.scenario.instructions.expected.as_u16())
    }
}

struct Scenario<'a> {
    endpoint: operation::Endpoint,
    instructions: mutation_instructions::MutationInstruction<'a>,
}

impl<'a> Scenario<'a> {
    fn new(
        endpoint: operation::Endpoint,
        instructions: mutation_instructions::MutationInstruction<'a>,
    ) -> Self {
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

    // Create endpoints from the spec file.
    let endpoints = spec.paths.iter().flat_map(|(path_name, methods)| {
        operation::Endpoint::create_supported_endpoint(path_name, methods.to_item_ref())
    });

    let scenarios = endpoints.fold(Vec::new(), |mut acc, endpoint| {
        let instructions = mutation_instructions::instructions_for_operation(endpoint.crud.clone());
        for instruction in instructions {
            acc.push(Scenario::new(endpoint.clone(), instruction));
        }
        acc
    });

    let scenario_executions = scenarios.into_iter().map(|scenario| {
        let request_path = mutator.make_path(&scenario.endpoint.path_name);
        let request_parameters = match mutator.make_query_params(
            &scenario.endpoint.method,
            &scenario.instructions.query_params,
        ) {
            // No valid query params could be created to fulfill this mutation. This is a bug.
            // TODO: Conversion of two ways of doing request params
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

        match validator::validate(&real_response, execution.expected_status_code(), execution.expected_body()) {
            Err(error) => {
                println!("{:?}", execution.scenario.endpoint);
                println!("{}", error.to_string());
                cli::print_error("Test failed.");
            }
            Ok(_) => cli::print_success("Test passed."),
        }

        println!();
    }
}
