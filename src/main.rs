//mod string_validator;
mod cli;
mod cli_args;
mod error;
mod known_param;
mod mutation_instructions;
mod mutator;
mod operation;
mod request_param;
mod scenario;
mod service;
mod spec;
mod validator;

use crate::service::Service;
use openapi_utils::{ReferenceOrExt, ServerExt, SpecExt};
use scenario::Scenario;
use std::time::Instant;

fn main() {
    let config = cli_args::config();
    let spec = spec::read(&config.filename).deref_all();
    let service = Service::new(&config, spec.servers[0].base_path());
    let mutator = mutator::Mutator::new();

    // Create endpoints from the spec file.
    let endpoints = spec.paths.iter().flat_map(|(path_name, methods)| {
        operation::Endpoint::create_supported_endpoint(path_name, methods.to_item_ref())
    });

    // TODO: Clean up all this Scenario stuff
    let scenarios = endpoints.fold(Vec::new(), |mut acc, endpoint| {
        for instruction in mutation_instructions::mutations() {
            acc.push(Scenario::new(endpoint.clone(), instruction));
        }
        acc
    });

    let scenario_executions = scenarios.into_iter().map(|scenario| {
        let request = mutator.request(&scenario.endpoint, &scenario.instructions);
        scenario::ScenarioExecution { scenario, request }
    });

    //     let scenario_executions = Vec::new();
    //     for endpoint in endpoints {
    //         for instruction in mutation_instructions::mutations() {
    //             let request = mutator.request(&endpoint, &instruction);
    //  //           cli::print_mutation_scenario(&endpoint.path_name, &instruction);
    //             scenario_executions.push(scenario::ScenarioExecution { scenario, request });
    //         }
    //     }

    //    println!("{:?}", scenario_executions);

    // TODO Report why not runable if user can do something (add to conversions)
    let (runable, not_runable): (
        Vec<scenario::ScenarioExecution>,
        Vec<scenario::ScenarioExecution>,
    ) = scenario_executions.partition(|s| s.request.is_some());

    let start = Instant::now();
    // Run each scenario execution, get the response and validate it with what we expect
    for execution in &runable {
        cli::print_mutation_scenario(
            &execution.scenario.endpoint,
            &execution.scenario.instructions,
        );

        let response = service.send(&execution.request.as_ref().unwrap());

        match response {
            Err(e) => {
                println!("Problem connecting to the service under test.\n {}", e);
                cli::print_error("Test failed.");
            }
            Ok(real_response) => {
                match validator::validate(
                    &real_response,
                    execution.expected_status_code(),
                    execution.expected_body(),
                ) {
                    Err(error) => {
                        //println!("{:?}", execution.scenario.endpoint);
                        println!("{}", error.to_string());
                        cli::print_error("Test failed.");
                    }
                    Ok(_) => cli::print_success("Test passed."),
                }
            }
        }
        println!();
    }

    // TODO: Output if we failed or all passed
    println!(
        "Executed {} scenarios in {:?}",
        runable.len(),
        start.elapsed()
    );
}
