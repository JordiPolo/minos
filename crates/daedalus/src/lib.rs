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

// TODO: add base path
//spec.servers[0].base_path()
pub struct GeneratorConfig {
    filename: String,
    conv_filename: String,
    scenarios_all_codes: bool,
    matches: String,
}

impl GeneratorConfig {
    pub fn new(
        filename: String,
        conv_filename: String,
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

pub struct Generator {
    mutator: mutation::Mutator,
    endpoints: Vec<operation::Endpoint>,
}
impl Generator {
    pub fn new(config: &GeneratorConfig) -> Self {
        let spec = spec::read(&config.filename).deref_all();
        let mutator = mutation::Mutator::new(&config.conv_filename, config.scenarios_all_codes);
        let endpoints = Self::endpoints(spec, &config.matches);
        Generator { mutator, endpoints }
    }

    pub fn scenarios<'a>(&'a self) -> impl Iterator<Item = crate::scenario::Scenario<'a>> {
        self.endpoints
            .iter()
            .flat_map(move |e| self.mutator.mutate(&e))
    }

    fn endpoints(spec: openapiv3::OpenAPI, matches: &str) -> Vec<operation::Endpoint> {
        let base_path = spec.servers[0].base_path();
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
