use crate::reporter;
use daedalus::Scenario;
use tracing::debug;

pub fn run<'a>(scenarios: impl Iterator<Item = Scenario<'a>>) {
    let mut total = 0;
    for scenario in scenarios {
        debug!("{:?}", scenario.request());
        reporter::print_mutation_scenario(&scenario);
        total += 1;
    }
    println!("{:?} scenarios generated.", total);
}
