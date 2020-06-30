use crate::known_param::KnownParamCollection;
use crate::mutation::bool_type;
use crate::mutation::integer_type;
use crate::mutation::param_mutation::ParamMutation;
use crate::mutation::string_type;
use crate::mutation::Mutagen;
use openapi_utils::{ParameterDataExt, ParameterExt};
use openapiv3::Type;

/// TODO: How to make sure we generate for all the Mutagens?
pub fn mutate(
    param: &openapiv3::Parameter,
    known_params: &KnownParamCollection,
    run_all_scenarios: bool,
) -> ParamMutation {
    let data = param.parameter_data();
    let mut mutations = ParamMutation::new_param(param);

    mutations.push_multiple(None, Mutagen::None, param.parameter_data().required);

    if known_params.param_known(&data.name) {
        mutations.push(&known_params.param_value(&data.name), Mutagen::ParamProper);
        if !run_all_scenarios {
            return mutations; // As soon as we have something that can give 200, give up trying to make more scenarios
        }
    }

    if !data.is_type_defined() {
        //return empty, we can't mutate anything without types
        return ParamMutation::new_param(param);
    } else {
        let typed_mutations = match data.get_type() {
            Type::Boolean {} => bool_type::mutate(&param),
            Type::Integer(the_type) => integer_type::mutate(&param, the_type),
            //Type::Number { .. } => {
            //     RequestParam::new(&data.name, "NotAnIntegerhahahaha")
            // },
            // Type::Array { .. } => RequestParam::new(&data.name, "notAnArray"),
            // Type::Object { .. } => RequestParam::new(&data.name, "notAnObject"),
            Type::String(the_type) => string_type::mutate(&param, the_type),
            _ => ParamMutation::new_param(param), //unimplemented!("Not this umproper"),
        };
        mutations.extend(typed_mutations);
    }
    mutations
}
