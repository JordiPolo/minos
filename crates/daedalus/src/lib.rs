/*!
The Daedalus crate provides auto-generation of test scenarios out of openapi files.
*/

#![deny(missing_docs)]

mod error;
mod known_param;
mod mutation;
mod operation;
mod request;
mod request_param;
mod scenario;
mod spec;

use openapi_utils::{ReferenceOrExt, ServerExt, SpecExt};

pub use request::Request;
pub use scenario::Scenario;
pub use scenario::ScenarioExpectation;

/// Configuration for the generation of the scenarios.
pub struct GeneratorConfig {
    /// The file with the openapi specification
    filename: String,
    /// The file with variable information to fill in the scenarios
    conv_filename: Option<String>,
    /// If we want to generate scenarios for all codes including failures
    scenarios_all_codes: bool,
    /// Regex string. Only scenarios for paths matching this string will be generated
    matches: String,
}

impl GeneratorConfig {
    /// Constructor for configuration
    pub fn new(
        filename: String,
        conv_filename: Option<String>,
        scenarios_all_codes: bool,
        matches: String,
    ) -> Self {
        GeneratorConfig {
            filename,
            conv_filename,
            scenarios_all_codes,
            matches,
        }
    }
}

/// This is the builder object in this library it creates the scenarios you can work with.
pub struct Generator {
    mutator: mutation::Mutator,
    endpoints: Vec<operation::Endpoint>,
}
impl Generator {
    /// Construction out of the configuration
    pub fn new(config: &GeneratorConfig) -> Result<Self, error::DaedalusError> {
        let spec: openapiv3::OpenAPI = spec::read(&config.filename)?;
        let spec = spec.deref_all();
        let mutator = mutation::Mutator::new(&config.conv_filename, config.scenarios_all_codes)?;
        let endpoints = Self::endpoints(spec, &config.matches);
        Ok(Generator { mutator, endpoints })
    }

    /// Returns an iterator over the scenarios generated by the generator
    pub fn scenarios(&self) -> impl Iterator<Item = crate::scenario::Scenario> {
        self.endpoints
            .iter()
            .flat_map(move |e| self.mutator.mutate(&e))
    }

    fn endpoints(spec: openapiv3::OpenAPI, matches: &str) -> Vec<operation::Endpoint> {
        let base_path = match spec.servers.first() {
            None => String::from(""),
            Some(server) => server.base_path(),
        };
        spec.paths
            .into_iter()
            .filter(|p| p.0.contains(&matches))
            .flat_map(|(path_name, methods)| {
                operation::Endpoint::new_supported(
                    &format!("{}{}", base_path, path_name),
                    methods.to_item_ref(),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn passing_inexisting_files() {
        let config = GeneratorConfig::new(
            "not_there.yaml".to_string(),
            Some("not_there_either.yaml".to_string()),
            true,
            "/".to_string(),
        );
        Generator::new(&config).unwrap();
    }

    #[test]
    fn passing_existing_files() {
        let config = GeneratorConfig::new(
            "test_openapi.yaml".to_string(),
            Some("test_conversions.yaml".to_string()),
            true,
            "/".to_string(),
        );
        let generator = Generator::new(&config).unwrap();
        let scenarios = generator.scenarios();
        let mut ein = 0;
        for scenario in scenarios {
            ein = ein + 1;
        }
        assert_eq!(ein, 5);
    }
}
