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

pub fn run(scenarios: &Vec<scenario::Scenario<'_>>, service: &service::Service, users: usize) {
    // TODO: I need to put all the defaults here because the method that set them up also sets stuff I do not want.
    let mut configuration: GooseConfiguration = Default::default();
    configuration.hatch_rate = 1; // Make one user per second
    configuration.users = Some(users); // How many users accessing simultaneously
    configuration.run_time = (users + 60).to_string(); // End test after 1 minute after users all online
    configuration.reset_stats = true; // Stats reset after hatching is completed so only count final 1 minute
    configuration.status_codes = false; // Add or not stats about status codes
    configuration.stats_log_format = "json".to_string();
    configuration.log_file = "/tmp/minos_performance".to_string();
    configuration.debug_log_format = "json".to_string();
    configuration.debug_log_file = "/tmp/minos_performance".to_string();
    configuration.host = "http://localhost:3000".to_string(); // Dummy. Goose needs this. we overwrite urls later
    let goose_attack = GooseAttack::initialize_with_config(configuration).setup();

    {
        // Need to drop the rwlock after this block so we can read it
        let mut req_list = PATH_HEADER_LIST.write().unwrap();
        for scenario in scenarios {
            let request = service.build_hyper_request(&scenario.request);
            let url = reqwest::Url::parse(&request.uri().to_string()).unwrap();
            req_list.push((url, request.headers().clone()));
        }
    }

    println!("Running performance test for 1 minute. Ramping up and gathering statistics...");
    //goose_attack
    goose_attack
        .register_taskset(taskset!("Minos Performance Test").register_task(task!(call_url)))
        .execute();
}

async fn call_url(user: &GooseUser) {
    let path_header_list = PATH_HEADER_LIST.read().unwrap().clone();
    for path_header in path_header_list {
        let builder = user
            .client
            .lock()
            .await
            .get(path_header.0)
            .headers(path_header.1);
        user.goose_send(builder, None).await;
    }
}
