use crate::operation::Endpoint;
use std::fmt::Display;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::error::Disparity;
use crate::mutation::instructions;
use crate::mutation::instructions::MutationInstruction;
use crate::scenario::ScenarioExecution;

pub fn connection_error(e: reqwest::Error) {
    println!("Problem connecting to the service under test.\n {}", e);
    print_error("Test failed.");
}

pub fn test_failed(error: Disparity) {
    println!("{}", error.to_string());
    print_error("Test failed.");
}

pub fn test_passed() {
    print_success("Test passed.");
}

pub fn print_mutation_scenario(endpoint: &Endpoint, mutation: &MutationInstruction) {
    print_scenario("Scenario:");
    print_scenario(format!(
        "{} {}",
        endpoint.crud.to_method_name(),
        endpoint.path_name
    ));
    print_scenario(format!("  {}", mutation.explanation));
    print_scenario(format!("  Expects {}", mutation.expected));
}


pub fn advise_about_conversions_file(not_runable: &Vec<ScenarioExecution>) {
    let mut convertible: Vec<String> = not_runable.iter().filter_map(|execution|
        if execution.scenario.instructions.path_params == instructions::PathMutation::Proper &&
        execution.scenario.instructions.required_params == instructions::ParamMutation::Proper &&
        execution.scenario.instructions.query_params == instructions::ParamMutation::Proper
         {
            Some(execution.scenario.endpoint.path_name.clone())
        } else { None }
    ).collect();
    convertible.dedup();

    if !convertible.is_empty() {
        println!();
        println!("Hint: Include known path IDs to the conversions file to increase the coverage of Minos for the following paths:");
        convertible.iter().for_each(|path| println!("{:?}", path));
    }

}

pub fn run_summary(runable_executions: &Vec<ScenarioExecution>, start: std::time::Instant) {
    let failed = runable_executions
        .iter()
        .filter(|&x| x.passed == false)
        .count();

    println!(
        "{} scenarios executed in {:?}.\n {:?} passed, {:?} failed.",
        runable_executions.len(),
        start.elapsed(),
        runable_executions.len() - failed,
        failed,
    );

    if failed > 0 {
        print_error("Some tests have failed.");
    }
}

fn print_success(message: impl Display) {
    print_color(message, Color::Green);
}

fn print_error(error: impl Display) {
    print_color(error, Color::Red);
}

fn print_scenario(message: impl Display) {
    print_color(message, Color::Blue);
}

fn print_color(error: impl Display, color: Color) {
    let mut output = StandardStream::stdout(ColorChoice::Auto);
    output
        .set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))
        .unwrap();
    writeln!(output, "{}", error).unwrap();
    output.reset().unwrap();
}
