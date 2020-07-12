use crate::reporter;
use crate::scenario::Scenario;
use crate::service;
use crate::validator;
use log::debug;
use std::time::Instant;

pub fn run<'a>(
    scenarios: impl Iterator<Item = Scenario<'a>>,
    service: &service::Service,
    allow_missing_rs: bool,
) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let start = Instant::now();
    let results = rt.block_on(run_testing_scenarios(scenarios, &service, allow_missing_rs));
    reporter::run_summary(&results, start);
}

async fn run_testing_scenarios<'a>(
    scenarios: impl Iterator<Item = Scenario<'a>>,
    service: &service::Service,
    allow_missing_rs: bool,
) -> Vec<(String, bool)> {
    let mut results = Vec::new();

    for scenario in scenarios {
        let path = scenario.endpoint.path_name.clone();
        println!("{:?}", scenario.request.to_string());

        reporter::print_mutation_scenario(&scenario);

        let response = service.send(&scenario.request).await;

        match response {
            Err(e) => {
                reporter::connection_error(e);
                results.push((path, false));
            }
            Ok(real_response) => {
                match validator::validate(real_response, scenario.expectation(), allow_missing_rs) {
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
