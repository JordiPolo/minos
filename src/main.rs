#![deny(clippy::all)]
#![forbid(unsafe_code)]

mod authentication;
mod cli_args;
mod command_ls;
mod command_performance;
mod command_verify;
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

use crate::cli_args::Command;
use crate::service::Service;
use openapi_utils::{ReferenceOrExt, ServerExt, SpecExt};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::FmtSubscriber;

const LOGO: &str = r"
           _
          (_)
 _ __ ___  _ _ __   ___   ___
| '_ ` _ \| | '_ \ / _ \ / __|
| | | | | | | | | | (_) |\__ \
|_| |_| |_|_|_| |_|\___/ \___/
";

fn main() {
    FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_env("MINOS_LOG"))
        .without_time()
        .init();

    let config = cli_args::config();
    let spec = spec::read(&config.filename).deref_all();
    let mutator = mutation::Mutator::new(&config.conv_filename, config.scenarios_all_codes);
    let service = Service::new(&config, spec.servers[0].base_path());
    println!("{}", LOGO);

    let endpoints  : Vec<operation::Endpoint> = spec
        .paths
        .iter()
        .filter(|p| p.0.contains(&config.matches))
        .flat_map(|(path_name, methods)| {
            operation::Endpoint::new_supported(path_name, methods.to_item_ref())
        }).collect();

    let scenarios = endpoints.iter().flat_map(|e| mutator.mutate(&e));

    match config.command {
        Command::Ls => command_ls::run(scenarios),
        Command::Performance(config) => command_performance::run(scenarios, &service, config),
        Command::Verify { without_rs } => command_verify::run(scenarios, &service, without_rs),
    }
}
