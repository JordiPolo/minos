use crate::mutation::param_mutation::ParamMutation;
use crate::mutation::Mutagen;
use openapi_utils::ParameterExt;

pub fn mutate(param: &openapiv3::Parameter) -> ParamMutation {
    let mut mutations = ParamMutation::new_param(&param);
    mutations.push("false", Mutagen::ParamProper);
    if param.parameter_data().name != "include_count" {
        mutations.push("NotABool", Mutagen::WrongPattern);
    }
    mutations
}
