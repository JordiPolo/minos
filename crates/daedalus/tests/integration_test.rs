use daedalus::*;
use daedalus::Scenario;


fn test_support() -> std::path::PathBuf  {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // chop off exe name
    path.pop(); // chop off 'deps'
    path.pop(); // chop off 'debug'
    path.pop(); // chop off 'target'

    // If `cargo test` is run manually then our path looks like
    // `target/debug/foo`, in which case our `path` is already pointing at
    // `target`. If, however, `cargo test --target $target` is used then the
    // output is `target/$target/debug/foo`, so our path is pointing at
    // `target/$target`. Here we conditionally pop the `$target` name.
    // if path.file_name().and_then(|s| s.to_str()) != Some("target") {
    //     path.pop();
    // }

    path.push("tests");
    path.push("support");
    path
}

fn support_file(filename: &str) -> String {
    let mut name = test_support();
    name.push(filename);
    name.into_os_string().into_string().unwrap()
}

#[test]
fn reading_inexisting_files_fails() {
    let config = GeneratorConfig::new(
        "not_there.yaml".to_string(),
        Some("not_there_either.yaml".to_string()),
        true,
        "/".to_string(),
    );
    let generator = Generator::new(&config);
    assert_eq!(generator.is_err(), true);
}

#[test]
fn without_conversions_default_scenarios() {
    let config = GeneratorConfig::new(
        support_file("test_openapi.yaml"),
        None,
        true,
        "/".to_string(),
    );
    let generator = Generator::new(&config).unwrap();
    let scenarios: Vec<Scenario>  = generator.scenarios().collect();

    assert_eq!(scenarios.len(), 5);
    assert_eq!(scenarios[0].expectation().status_code, 200);
    assert_eq!(scenarios[1].expectation().status_code, 406);
    assert_eq!(scenarios[2].expectation().status_code, 200);
    assert_eq!(scenarios[3].expectation().status_code, 200);
    assert_eq!(scenarios[4].expectation().status_code, 404);
}

#[test]
fn without_conversions_without_errors_default_scenarios() {
    let config = GeneratorConfig::new(
        support_file("test_openapi.yaml"),
        None,
        false,
        "/".to_string(),
    );
    let generator = Generator::new(&config).unwrap();
    let scenarios: Vec<Scenario>  = generator.scenarios().collect();

    assert_eq!(scenarios.len(), 3);
    assert_eq!(scenarios[0].expectation().status_code, 200);
    assert_eq!(scenarios[1].expectation().status_code, 200);
    assert_eq!(scenarios[2].expectation().status_code, 200);
}

#[test]
fn passing_existing_files() {
    let config = GeneratorConfig::new(
        support_file("test_openapi.yaml"),
        Some(support_file("test_conversions.yaml")),
        true,
        "/".to_string(),
    );
    let generator = Generator::new(&config).unwrap();
    let scenarios: Vec<Scenario>  = generator.scenarios().collect();
    assert_eq!(scenarios[0].instructions.len(), 5);

    assert_eq!(scenarios.len(), 7);
    assert_eq!(scenarios[0].expectation().status_code, 200);
    assert_eq!(scenarios[1].expectation().status_code, 406);
    assert_eq!(scenarios[2].expectation().status_code, 200);
    assert_eq!(scenarios[3].expectation().status_code, 200);
    assert_eq!(scenarios[4].expectation().status_code, 200);
    assert_eq!(scenarios[5].expectation().status_code, 404);
    assert_eq!(scenarios[6].expectation().status_code, 406);
}
