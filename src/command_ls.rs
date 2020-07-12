use crate::reporter;
use crate::scenario::Scenario;

pub fn run<'a>(scenarios: impl Iterator<Item = Scenario<'a>>) {
    let mut total = 0;
    for scenario in scenarios {
        println!("{:?}", scenario.request.to_string());
        reporter::print_mutation_scenario(&scenario);
        total += 1;
    }
    println!("{:?} scenarios generated.", total);
}
