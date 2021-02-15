#![deny(clippy::all)]
#![forbid(unsafe_code)]

mod authentication;
mod cli_args;
mod command_ls;
mod command_performance;
mod command_verify;
mod error;
mod reporter;
mod service;
mod validator;

use crate::cli_args::Command;
use crate::service::Service;
use daedalus::Generator;
use anyhow::{Context, Result};
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

fn main() -> Result<()>{
    FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_env("MINOS_LOG"))
        .without_time()
        .init();

    let config = cli_args::config();
    let service = Service::new(&config.base_url);

    let generator = Generator::new(&config.generator_config()).context("Impossible to generate scenarios")?;
    let scenarios = generator.scenarios();

    println!("{}", LOGO);
    match config.command {
        Command::Ls => command_ls::run(scenarios),
        Command::Performance(config) => command_performance::run(scenarios, &service, config),
        Command::Verify { without_rs } => command_verify::run(scenarios, &service, without_rs),
    }
    Ok(())
}
