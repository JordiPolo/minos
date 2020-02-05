//mod string_validator;
#![deny(clippy::all)]
#![forbid(unsafe_code)]

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

use log::debug;
use openapi_utils::{ReferenceOrExt, ServerExt, SpecExt};
use std::time::Instant;

use crate::service::Service;
fn main() {
    env_logger::init();

    let config = cli_args::config();
    let spec = spec::read(&config.filename).deref_all();
    let service = Service::new(&config, spec.servers[0].base_path());
    let mutator = mutation::Mutator::new(&config.conv_filename);
    let mut results = Vec::new();


    // Create endpoints from the spec file.
    let endpoints: Vec<operation::Endpoint> = spec.paths.iter().flat_map(|(path_name, methods)| {
        operation::Endpoint::create_supported_endpoint(path_name, methods.to_item_ref())
    }).collect();

    let scenarios = endpoints.iter().flat_map(|e| mutator.mutate(e));

    let start = Instant::now();
    for scenario in scenarios {
        //  println!("{:?}", scenario.instructions);
        reporter::print_mutation_scenario(&scenario);
        let path = scenario.endpoint.path_name.clone();
        if config.dry_run {
            println!("Dry run mode. No request executed.\n");
            results.push((path, true));
            continue;
        }

        let response = service.send(&scenario.request);

        match response {
            Err(e) => {
                reporter::connection_error(e);
                results.push((path, false));
            }
            Ok(real_response) => {
                match validator::validate(real_response, scenario.expectation()) {
                    Err(error) => {
                        debug!("{:?}", scenario.endpoint);
                        reporter::test_failed(error);
                        results.push((path, false));
                    }
                    Ok(_) => {
                        // variation.passed = true;
                        reporter::test_passed();
                        results.push((path, true));
                    }
                }
            }
        }
        println!();
    }

    reporter::run_summary(&results, start);
}
