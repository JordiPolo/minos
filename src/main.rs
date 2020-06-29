//mod string_validator;
#![deny(clippy::all)]
#![forbid(unsafe_code)]

mod cli_args;
mod error;
mod known_param;
mod mutation;
mod operation;
mod performance;
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


async fn run_testing_scenarios(scenarios: &Vec<scenario::Scenario<'_>>, config: &cli_args::CLIArgs, spec: &openapiv3::OpenAPI) -> Vec<(String, bool)> {
    let service = Service::new(&config, spec.servers[0].base_path());
    let mut results = Vec::new();

    for scenario in scenarios {
        let path = scenario.endpoint.path_name.clone();
        println!("{:?}", scenario.request.to_string());

        reporter::print_mutation_scenario(&scenario);
        if config.dry_run {
            println!("Dry run mode. No request executed.\n");
            results.push((path, true));
            continue;
        }

        let response = service.send(&scenario.request).await;

        match response {
            Err(e) => {
                reporter::connection_error(e);
                results.push((path, false));
            }
            Ok(real_response) => {
                match validator::validate(
                    real_response,
                    scenario.expectation(),
                    config.allow_missing_rs,
                ) {
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
    results
}


fn main() {
    env_logger::init();
    let config = cli_args::config();
    let spec = spec::read(&config.filename).deref_all();
    let mutator = mutation::Mutator::new(&config.conv_filename);
    let service = Service::new(&config, spec.servers[0].base_path());

    // Create endpoints from the spec file.
    let endpoints: Vec<operation::Endpoint> = spec
        .paths
        .iter()
        .filter(|p| p.0.contains(&config.matches))
        .flat_map(|(path_name, methods)| {
            operation::Endpoint::create_supported_endpoint(path_name, methods.to_item_ref())
        })
        .collect();

    let scenarios  = endpoints.iter()
        .flat_map(|e| mutator.mutate(e));

    if config.hammer_it {
        performance::run(&scenarios.collect(), &service, &config)
    } else {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let start = Instant::now();
        let results = rt.block_on(run_testing_scenarios(&scenarios.collect(), &config, &spec));
        reporter::run_summary(&results, start);
    }
}

