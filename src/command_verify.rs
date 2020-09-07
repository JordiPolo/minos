use crate::reporter;
use crate::service;
use crate::validator;
use daedalus::Scenario;
use std::time::Instant;
use tracing::debug;

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
        let request = scenario.request();
        let path = request.uri().path().to_owned();

        let runnable = service.runnable_request(request);
        reporter::print_runnable_scenario(&scenario, &runnable);
        let response = service.send(runnable).await;
        match response {
            Err(e) => {
                reporter::connection_error(e);
                results.push((path, false));
            }
            Ok(real_response) => {
                debug!("{:?}", &real_response.body);
                match validator::validate(real_response, scenario.expectation(), allow_missing_rs) {
                    Err(error) => {
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
