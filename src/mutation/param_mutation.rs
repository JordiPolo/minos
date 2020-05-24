use crate::mutation::instructions::schema_mutagen;
use crate::mutation::{Mutagen, Mutation};
use crate::request_param::RequestParam;
use openapi_utils::ParameterExt;
pub struct ParamMutation {
    pub variations: Vec<Mutation>,
    pub param: openapiv3::Parameter,
}

impl ParamMutation {
    pub fn new_param(param: &openapiv3::Parameter) -> Self {
        ParamMutation {
            variations: vec![],
            param: param.clone(),
        }
    }
    pub fn push(&mut self, value: &str, mutagen: Mutagen) {
        let instruction = schema_mutagen(&mutagen)[0].clone();
        let param = RequestParam::new(&self.param.parameter_data().name, value.clone());
        self.variations
            .push(Mutation::new_param(instruction, param));
    }
    pub fn push_multiple(&mut self, value: Option<String>, mutagen: Mutagen) {
        let instructions = schema_mutagen(&mutagen);
        for instruction in instructions {
            let param = RequestParam::new2(&self.param.parameter_data().name, value.clone());
            self.variations
                .push(Mutation::new_param(instruction, param));
        }
    }
    pub fn extend(&mut self, other: Self) {
        self.variations.extend(other.variations);
    }
}
