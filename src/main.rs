//mod string_validator;
#![deny(clippy::all)]

mod cli_args;
mod error;
mod known_param;
mod mutation;
mod operation;
mod reporter;
mod request_param;
mod scenario;
mod service;
mod spec;
mod validator;

use crate::service::Service;
use log::debug;
use openapi_utils::{ReferenceOrExt, ServerExt, SpecExt};
use scenario::{Scenario, ScenarioExecution};
use std::time::Instant;

fn main() {
    env_logger::init();

    let config = cli_args::config();
    let spec = spec::read(&config.filename).deref_all();
    let service = Service::new(&config, spec.servers[0].base_path());
    let mutator = mutation::Mutator::new(&config.conv_filename);

    // Create endpoints from the spec file.
    let endpoints = spec.paths.iter().flat_map(|(path_name, methods)| {
        operation::Endpoint::create_supported_endpoint(path_name, methods.to_item_ref())
    });

    // Scenarios from the endpoints
    let scenarios = endpoints.fold(Vec::new(), |mut acc, endpoint| {
        for instruction in mutation::instructions::mutations() {
            acc.push(Scenario::new(endpoint.clone(), instruction));
        }
        acc
    });

    // Runable executions for the scenarios
    let (mut runable_executions, not_runable): (Vec<ScenarioExecution>, Vec<ScenarioExecution>) =
        scenarios
            .into_iter()
            .map(|scenario| {
                let request = mutator.request(&scenario.endpoint, &scenario.instructions);
                ScenarioExecution::new(scenario, request)
            })
            .partition(|s| s.request.is_some());

    let start = Instant::now();
    // Run each scenario execution, get the response and validate it
    for execution in &mut runable_executions {
        reporter::print_mutation_scenario(
            &execution.scenario.endpoint,
            &execution.scenario.instructions,
        );

        let response = service.send(&execution.request.as_ref().unwrap());

        match response {
            Err(e) => {
                reporter::connection_error(e);
            }
            Ok(real_response) => {
                match validator::validate(real_response, execution.scenario.expectation()) {
                    Err(error) => {
                        debug!("{:?}", execution.scenario.endpoint);
                        reporter::test_failed(error);
                    }
                    Ok(_) => {
                        execution.passed = true;
                        reporter::test_passed();
                    }
                }
            }
        }
        println!();
    }

    // for r in not_runable {
    //     info!("{:?} ---> {:?}", r.scenario.endpoint.path_name, r.scenario.instructions);
    // }

    reporter::run_summary(&runable_executions, start);
    reporter::advise_about_conversions_file(&not_runable);
}
