use openapi;
use std::string::String;

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
            .unwrap()
            .get(&self.json_ref_name(&definition_name))
            .unwrap()
            .clone()
    }

    pub fn json_ref_name(&self, reference: &str) -> String {
        reference.split("/").last().unwrap().to_owned()
    }

    pub fn resolve_parameter(&self, parameter_name: &str) -> openapi::v2::Parameter {
        let global_params = self.spec.parameters.clone().unwrap();
        global_params
            .get(&self.json_ref_name(&parameter_name))
            .unwrap()
            .clone()
    }
}

pub fn get_random_undefined_method(methods: &openapi::v2::PathItem) -> String {
    if methods.patch.is_none() {
        "patch".to_string()
    } else {
        "put".to_string()
    }
}

pub fn method_status_info(
    methods: &openapi::v2::PathItem,
    status_code: &str,
) -> Option<openapi::v2::Response> {
    let responses = methods.clone().get.map(|m| m.responses); //.unwrap();
    match responses {
        Some(resp) => resp.get(status_code).map(|a| a.clone()),
        None => None,
    }
    //responses.get(status_code).map(|a| a.clone())
}

pub fn extract_schema(status_info: &openapi::v2::Response) -> Option<openapi::v2::Schema> {
    status_info.clone().schema.clone() //and_then(|status| status.schema.clone())
}
