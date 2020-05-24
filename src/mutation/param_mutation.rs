use crate::mutation::{Mutagen, MutagenInstruction};
use crate::request_param::RequestParam;
use openapi_utils::ParameterExt;
use crate::mutation::instructions::schema_mutagen;
//use crate::mutation::Mutation;

pub struct ParamVariation {
    value: Option<String>,
    mutagen: MutagenInstruction,
}

impl ParamVariation {
    pub fn new(value: Option<String>, mutagen: MutagenInstruction) -> Self {
        ParamVariation {
            value,
            mutagen,
        }
    }
}

pub struct ParamMutation {
    variations: Vec<ParamVariation>,
    pub param: openapiv3::Parameter,
}

impl ParamMutation {
    pub fn to_params(&self) -> Vec<(RequestParam, MutagenInstruction)> {
        self.variations
            .iter()
            .map(|variation| {
                (
                    RequestParam::new2(&self.param.parameter_data().name, variation.value.clone()),
                    variation.mutagen.clone(),
                )
            })
            .collect()
    }

    pub fn new_param(param: &openapiv3::Parameter) -> Self {
        ParamMutation {
            variations: vec![],
            param: param.clone(),
        }
    }
    pub fn push(&mut self, value: &str, mutagen: Mutagen) {
        let instruction = schema_mutagen(&mutagen)[0].clone();
        self.variations.push(ParamVariation::new(Some(value.to_string()), instruction));
    }
    pub fn push_multiple(&mut self, value: Option<String>, mutagen: Mutagen) {
        let instructions = schema_mutagen(&mutagen);
        for instruction in instructions {
            self.variations.push(ParamVariation::new(value.clone(), instruction));
        }
    }
    pub fn extend(&mut self, other: Self) {
        self.variations.extend(other.variations);
    }
}
