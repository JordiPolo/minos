use crate::cli_args::PerformanceCommand;
use crate::reporter::print_value;
use crate::service;
use daedalus::Scenario;
use goose::prelude::*;
use goose::{GooseAttack, GooseConfiguration};
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderValue};
use std::sync::RwLock;
use std::{thread, time};
//use std::sync::Arc;

lazy_static! {
    static ref PATH_HEADER_LIST: RwLock<Vec<(reqwest::Url, HeaderMap<HeaderValue>)>> =
        RwLock::new(Vec::new());
}

pub fn run<'a>(
    scenarios: impl Iterator<Item = Scenario<'a>>,
    service: &service::Service,
    command: PerformanceCommand,
) {

    let config = config(command);
   //let goose_attack = default_attack(command);

    let goose_attack = GooseAttack::initialize_with_config(config.clone())
      // .setup()
       .expect("Failed to setup the performance test.");

    {
        // Need to drop the rwlock after this block so we can read it
        let mut req_list = PATH_HEADER_LIST.write().unwrap();
        for scenario in scenarios {
            let request = service.runnable_request(scenario.request()).http_request();
            let url = reqwest::Url::parse(&request.uri().to_string()).unwrap();
            req_list.push((url, request.headers().clone()));
        }
        if req_list.is_empty() {
            println!("There are 0 scenarios to run performance testing. Consider using a conversions file.");
            return;
        }
    }

    // let mut taskset = taskset!("WebsiteUser");
    // for scenario in scenarios {
    //     let request = service.runnable_request(scenario.request()).http_request();
    //     //let url = reqwest::Url::parse(&request.uri().to_string()).unwrap();
    //     let url = Arc::new(request.uri().to_string()); //.clone();
    //     let headers = request.headers().clone();

    //     let closure: GooseTaskFunction = Arc::new(move |user| {

    //         Box::pin(async move {
    //             let builder = user
    //             .client
    //             .lock()
    //             .await
    //             .get(&*url.clone())
    //             .headers(headers.clone());
    //             user.goose_send(builder, None).await?;
    //             Ok(())
    //         })
    //     });
    //     let task = GooseTask::new(closure);
    //     let new_taskset = taskset.register_task(task);
    //     taskset = new_taskset;
    // }
    // let goose_stats = goose_attack
    //     .register_taskset(taskset)
    //     .execute()
    //     .expect("Errors while running performance test");



    let indicative = thread::spawn(move || {
        // sleep the hatching
       // let hatching : u32 = (config.users.unwrap() / (1 + config.hatch_rate.unwrap().parse().unwrap())) * 1000;
       let hatching = 1000; // TODO: this is not correct but hacky for now to make it compile
        thread::sleep(time::Duration::from_millis(hatching as u64));
       //let slot = parse_timespan("10s") * 10; // total time  * 1000 ms / 100 slots

        let slot = parse_timespan(&config.run_time) * 10; // total time  * 1000 ms / 100 slots
        let slot = slot as u64;
        let bar = ProgressBar::new(100);

        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:60.green/magenta} {percent:>0}% {msg}")
                .progress_chars("▌▌-"),
        );

        for _ in 0..100 {
            bar.inc(1);
            thread::sleep(time::Duration::from_millis(slot));
        }
        bar.finish();
        println!();
    });

    let goose_stats = goose_attack
        .register_taskset(taskset!("Minos Performance Test").register_task(task!(call_url)))
        .execute()
        .expect("Errors while running performance test");

    indicative.join().unwrap();

    goose_stats.print();
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
/*

fn default_attack(command: PerformanceCommand) -> goose::GooseAttack {
    let PerformanceCommand {
        users,
        request_per_second,
        time,
    } = command;
    let hatch_rate = users * 100;
    let empty_args: Vec<&str> = vec![];
    let configuration: GooseConfiguration = GooseConfiguration::parse_args_default(&empty_args).unwrap();
    goose::GooseAttack::initialize_with_config(configuration).unwrap()
    .set_default(GooseDefault::HatchRate, hatch_rate)
    .set_default(GooseDefault::Users, users)
    .set_default(GooseDefault::Host,  "http://localhost:3000")
    .set_default(GooseDefault::RunTime, parse_timespan(&time))
    .set_default(GooseDefault::StatusCodes, false)
    .set_default(GooseDefault::NoTaskMetrics, true)
    .set_default(GooseDefault::OnlySummary, false)
    .set_default(GooseDefault::MetricsFormat, "json")
    .set_default(GooseDefault::MetricsFile, "/tmp/minos_metrics")
    .set_default(GooseDefault::ThrottleRequests, request_per_second)
    .set_default(GooseDefault::StickyFollow, true)
    .set_default(GooseDefault::NoHashCheck, true)

    //attack
}
*/

use gumdrop::Options;

fn config(command: PerformanceCommand) -> GooseConfiguration {
    let PerformanceCommand {
        users,
        request_per_second,
        time,
    } = command;
    let hatch_rate = users * 10; // ramp up in 100 ms, hopefully it is possible.
    let empty_args: Vec<&str> = vec![];
    // ::default() is not working properly I think it is a bug inn structopt
    let mut configuration: GooseConfiguration = GooseConfiguration::parse_args_default(&empty_args).unwrap(); //default();
    configuration.hatch_rate = Some(hatch_rate.to_string()); // Make 32 users per second
    configuration.users = Some(users); // How many users accessing simultaneously
    configuration.run_time = time.clone(); // End test after time after users all online
    // No idea what the below does
    configuration.running_metrics = Some(0);
    configuration.status_codes = true; // Add or not stats about status codes
    configuration.report_file = "/tmp/minos.html".to_string();
                                       // Stuff I do not care about but need to be set for config not to blow up
    configuration.metrics_format = "json".to_string();
    configuration.requests_file = "/tmp/minos_metrics".to_string();
    configuration.log_file = "/tmp/minos_performance".to_string();
    configuration.report_file = "/tmp/minos.html".to_string();
    configuration.debug_format = "json".to_string();
    configuration.debug_file = "/tmp/minos_performance_debug".to_string();
    configuration.host = "http://localhost:3000".to_string(); // Dummy. Goose needs this. we overwrite urls later
    configuration.throttle_requests = request_per_second;
    configuration.no_task_metrics = true;

    print!("       Concurrent users: ");
    print_value(users);
    print!("Max requests per second: ");
    print_value(request_per_second);
    print!("               Duration: ");
    print_value(time);
    println!();

    configuration
}

// TODO: make code in Goose public and remove this
// Code lifted from the Goose library.
// Ideally we could access the already parsed information but it is private
// the module containting this is private also.

use regex::Regex;
use std::str::FromStr;

fn parse_timespan(time_str: &str) -> usize {
    match usize::from_str(time_str) {
        // If an integer is passed in, assume it's seconds
        Ok(t) => t,
        // Otherwise use a regex to extract hours, minutes and seconds from string.
        Err(_) => {
            let re = Regex::new(r"((?P<hours>\d+?)h)?((?P<minutes>\d+?)m)?((?P<seconds>\d+?)s)?")
                .unwrap();
            let time_matches = re.captures(time_str).unwrap();
            let hours = match time_matches.name("hours") {
                Some(_) => usize::from_str(&time_matches["hours"]).unwrap(),
                None => 0,
            };
            let minutes = match time_matches.name("minutes") {
                Some(_) => usize::from_str(&time_matches["minutes"]).unwrap(),
                None => 0,
            };
            let seconds = match time_matches.name("seconds") {
                Some(_) => usize::from_str(&time_matches["seconds"]).unwrap(),
                None => 0,
            };
            hours * 60 * 60 + minutes * 60 + seconds
        }
    }
}
