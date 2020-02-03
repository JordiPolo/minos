use std::fmt::Display;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::error::Disparity;
use crate::scenario::Scenario;

const LIGHT_BLUE: Color = Color::Rgb(150, 150, 255);
const BORING_GRAY: Color = Color::Rgb(119, 119, 119);

use comfy_table::Table;

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

pub fn print_mutation_scenario(scenario: &Scenario) {
    let mut printer = Printer::new();
    let endpoint = &scenario.endpoint;
    let mutations = &scenario.instructions;

    printer.print_scenario("Scenario:");
    printer.print_scenario(format!(
        "{} {}",
        endpoint.crud.to_method_name(),
        endpoint.path_name
    ));
    let mut sorted_mutations = mutations.to_owned();
    sorted_mutations.sort(); //_by(|a, b| a.mutagen.expected.cmp(&b.mutagen.expected));
    for mutation in &sorted_mutations {
        let color = if mutation.mutagen.expected == http::StatusCode::OK {
            LIGHT_BLUE
        } else {
            Color::Blue
        };
        printer.print_color(format!("  {}", mutation), color);
    }
    printer.print_color(
        format!(
            "  Expects {}",
            sorted_mutations.pop().unwrap().mutagen.expected
        ),
        Color::Blue,
    );
    printer.print_color(
        &format!("TraceId: {:?}", scenario.request.headers()["X-B3-TraceID"]),
        BORING_GRAY,
    );
}

use itertools::Itertools;

pub fn run_summary(results: &Vec<(String, bool)>, start: std::time::Instant) {
    let failed = results.iter().filter(|&x| x.1 == false).count();
    let by_path = results.iter().group_by(|x| &x.0);

    let mut table = Table::new();
    table.set_header(vec!["Path", "Scenarios run", "Scenarios passed"]);

    for (path, results) in &by_path {
        // let total = &results.into_iter().len();
        let (pfailed, ppassed): (Vec<&(String, bool)>, Vec<&(String, bool)>) =
            results.partition(|&x| x.1 == false);
        let the_size = pfailed.len() + ppassed.len();

        table.add_row(vec![
            path,
            &the_size.to_string(),
            &pfailed.len().to_string(),
        ]);
    }

    println!("{}", table);

    println!(
        "{} scenarios executed in {:?}.\n {:?} passed, {:?} failed.",
        results.len(),
        start.elapsed(),
        results.len() - failed,
        failed,
    );

    if failed > 0 {
        print_error("Some tests have failed.");
    } else {
        print_success("All test passed!");
    }
}

fn print_success(message: impl Display) {
    Printer::new().print_color(message, Color::Green);
}

fn print_error(error: impl Display) {
    Printer::new().print_color(error, Color::Red);
}

struct Printer {
    output: StandardStream,
}

impl Printer {
    fn new() -> Self {
        let output = StandardStream::stdout(ColorChoice::Auto);
        output.lock();
        Printer { output }
    }

    fn print_scenario(&mut self, message: impl Display) {
        self.print_color(message, Color::Blue);
    }

    fn print_color(&mut self, error: impl Display, color: Color) {
        self.output
            .set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))
            .unwrap();
        writeln!(self.output, "{}", error).unwrap();
        self.output.reset().unwrap();
    }
}
