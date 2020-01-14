use crate::operation::Endpoint;
use std::fmt::Display;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::mutation::instructions::MutationInstruction;
use crate::scenario::ScenarioExecution;

pub fn connection_error(e: reqwest::Error) {
    println!("Problem connecting to the service under test.\n {}", e);
    print_error("Test failed.");
}

pub fn test_failed(error: crate::error::DisparityError) {
    println!("{}", error.to_string());
    print_error("Test failed.");
}

pub fn test_passed() {
    print_success("Test passed.");
}

pub fn print_mutation_scenario(
    endpoint: &Endpoint,
    mutation: &MutationInstruction,
) {
    print_scenario("Scenario:");
    print_scenario(format!(
        "{} {}",
        endpoint.crud.to_method_name(),
        endpoint.path_name
    ));
    print_scenario(format!("  {}", mutation.explanation));
    print_scenario(format!("  Expects {}", mutation.expected));
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
