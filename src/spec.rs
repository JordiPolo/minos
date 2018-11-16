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
        self.spec.definitions.clone().unwrap()[&self.json_ref_name(&definition_name)].clone()
    }

    pub fn resolve_parameter_ref(
        &self,
        param_or_ref: &openapi::v2::ParameterOrRef,
    ) -> openapi::v2::Parameter {
        match param_or_ref.clone() {
            openapi::v2::ParameterOrRef::Parameter(openapi::v2::Parameter {
                name,
                location,
                required,
                schema,
                unique_items,
                param_type,
                format,
                description,
                minimum,
                maximum,
                default,
                enum_values,
                exclusive_minimum,
                exclusive_maximum,
                max_length,
                min_length,
                max_items,
                min_items,
                pattern,
            }) => openapi::v2::Parameter {
                name,
                location,
                required,
                schema,
                unique_items,
                param_type,
                format,
                description,
                minimum,
                maximum,
                default,
                enum_values,
                exclusive_minimum,
                exclusive_maximum,
                max_length,
                min_length,
                max_items,
                min_items,
                pattern,
            },
            openapi::v2::ParameterOrRef::Ref { ref_path } => self.resolve_parameter(&ref_path),
        }
    }

    //TODO: this does not need the spec, move somewhere else?
    pub fn json_ref_name(&self, reference: &str) -> String {
        reference.split('/').last().unwrap().to_owned()
    }

    fn resolve_parameter(&self, parameter_name: &str) -> openapi::v2::Parameter {
        let global_params = self.spec.parameters.clone().unwrap();
        global_params[&self.json_ref_name(&parameter_name)].clone()
    }
}
