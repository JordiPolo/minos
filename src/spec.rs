use openapi;
use std::string::String;

#[derive(Clone)]
pub struct Spec {
    pub spec: openapi::v2::Spec,
}

impl Spec {
    pub fn from_filename(filename: &str) -> Self {
        let spec = match openapi::from_path(filename).unwrap() {
            openapi::OpenApi::V2(spec) => spec,
            openapi::OpenApi::V3_0(_) => {
                panic!("{:?}", "Version 3.0 of the OpenAPI spec not supported")
            }
        };
        Spec { spec }
    }

    pub fn resolve_definition(&self, definition_name: &str) -> openapi::v2::Schema {
        self.spec
            .definitions
            .clone()
            .unwrap()[&self.json_ref_name(&definition_name)]
            .clone()
    }

    //TODO: this does not need the spec, move somewhere else?
    pub fn json_ref_name(&self, reference: &str) -> String {
        reference.split('/').last().unwrap().to_owned()
    }

    pub fn resolve_parameter(&self, parameter_name: &str) -> openapi::v2::Parameter {
        let global_params = self.spec.parameters.clone().unwrap();
        global_params[&self.json_ref_name(&parameter_name)].clone()
    }
}

// pub fn get_random_undefined_method(methods: &openapi::v2::PathItem) -> String {
//     if methods.patch.is_none() {
//         "patch".to_string()
//     } else {
//         "put".to_string()
//     }
// }
