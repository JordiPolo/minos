use crate::mutation::Mutagen;
use crate::request_param::RequestParam;
use openapi_utils::ParameterExt;

pub struct ParamVariation {
    value: String,
    mutagen: Mutagen,
}

impl ParamVariation {
    pub fn new(value: &str, mutagen: Mutagen) -> Self {
        ParamVariation {
            value: String::from(value),
            mutagen,
        }
    }
}

pub struct ParamMutation {
    variations: Vec<ParamVariation>,
    name: String,
}

impl ParamMutation {
    pub fn to_params(&self) -> Vec<(RequestParam, Mutagen)> {
        self.variations
            .iter()
            .map(|variation| {
                (
                    RequestParam::new(&self.name, &variation.value),
                    variation.mutagen.clone(),
                )
            })
            .collect()
    }
    pub fn new(name: &str) -> Self {
        ParamMutation {
            variations: vec![],
            name: String::from(name),
        }
    }

    pub fn new_param(param: &openapiv3::Parameter) -> Self {
        ParamMutation {
            variations: vec![],
            name: String::from(&param.parameter_data().name),
        }
    }
    pub fn push(&mut self, value: &str, mutagen: Mutagen) {
        self.variations.push(ParamVariation::new(value, mutagen));
    }
    pub fn extend(&mut self, other: Self) {
        self.variations.extend(other.variations);
    }
}
