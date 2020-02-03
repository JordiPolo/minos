use crate::mutation::param_mutation::ParamMutation;
use crate::mutation::Mutagen;

pub fn mutate(param: &openapiv3::Parameter) -> ParamMutation {
    let mut mutations = ParamMutation::new_param(&param);
    mutations.push("false", Mutagen::ParamProper);
    mutations.push("NotABool", Mutagen::WrongPattern);
    mutations
}
