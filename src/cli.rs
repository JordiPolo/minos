use std::fmt::Display;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use mutation;

pub fn print_mutation_scenario(path_name: &str, mutation: &mutation::Mutation) {
    print_scenario("Scenario:");
    print_scenario(path_name);
    print_scenario(format!("  {}", mutation.explanation));
    print_scenario(format!("  Expects {}", mutation.expected));
}


pub fn print_success(message: impl Display) {
    print_color(message, Color::Green);
}

pub fn print_error(error: impl Display) {
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
    writeln!(output, "{}", error);
    output.reset().unwrap();
}

