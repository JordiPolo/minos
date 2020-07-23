use crate::cli_args::PerformanceCommand;
use crate::scenario;
use crate::service;
use goose::prelude::*;
use goose::{GooseAttack, GooseConfiguration};
use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderValue};
use std::sync::RwLock;

lazy_static! {
    static ref PATH_HEADER_LIST: RwLock<Vec<(reqwest::Url, HeaderMap<HeaderValue>)>> =
        RwLock::new(Vec::new());
}

pub fn run<'a>(
    scenarios: impl Iterator<Item = scenario::Scenario<'a>>,
    service: &service::Service,
    command: PerformanceCommand,
) {
    let goose_attack = GooseAttack::initialize_with_config(config(command))
        .setup()
        .expect("Failed to setup the performance test.");

    {
        // Need to drop the rwlock after this block so we can read it
        let mut req_list = PATH_HEADER_LIST.write().unwrap();
        for scenario in scenarios {
            let request = service.build_hyper_request(&scenario.request);
            let url = reqwest::Url::parse(&request.uri().to_string()).unwrap();
            req_list.push((url, request.headers().clone()));
        }
    }

    println!("Running performance test. Ramping up and gathering statistics...");

    let goose_values = goose_attack
        .register_taskset(taskset!("Minos Performance Test").register_task(task!(call_url)))
        .execute()
        .expect("Errors while running performance test");

    println!("\n\n\n< ----------------------------------------------------------------- >");
    println!("< ---------------------- Aggregated Stats ------------------------- >");
    println!("< ----------------------------------------------------------------- >");
    goose_values.display();
}

async fn call_url(user: &GooseUser) -> GooseTaskResult {
    let path_header_list = PATH_HEADER_LIST.read().unwrap().clone();
    for path_header in path_header_list {
        let builder = user
            .client
            .lock()
            .await
            .get(path_header.0)
            .headers(path_header.1);
        user.goose_send(builder, None).await?;
    }
    Ok(())
}

fn config(command: PerformanceCommand) -> GooseConfiguration {
    let PerformanceCommand {
        users,
        request_per_second,
        time,
    } = command;
    let hatch_rate = 32;
    //let total_time = (users/hatch_rate  + time) as usize;

    // ::default() is not working properly I think it is a bug inn structopt
    let mut configuration: GooseConfiguration = GooseConfiguration::default();
    configuration.hatch_rate = hatch_rate; // Make 32 users per second
    configuration.users = Some(users); // How many users accessing simultaneously
    configuration.run_time = time; // End test after time after users all online
    configuration.reset_stats = true; // Stats reset after hatching is completed so only count working hard
    configuration.status_codes = false; // Add or not stats about status codes
    // Stuff I do not care about but need to be set for config not to blow up
    configuration.stats_log_format = "json".to_string();
    configuration.log_file = "/tmp/minos_performance".to_string();
    configuration.debug_log_format = "json".to_string();
    configuration.debug_log_file = "/tmp/minos_performance".to_string();
    configuration.host = "http://localhost:3000".to_string(); // Dummy. Goose needs this. we overwrite urls later
    configuration.throttle_requests = Some(request_per_second);
    configuration
}
