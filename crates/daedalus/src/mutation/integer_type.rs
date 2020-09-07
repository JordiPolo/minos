use crate::mutation::param_mutation::ParamMutation;
use crate::mutation::Mutagen;
use openapi_utils::{IntegerTypeExt, ParameterExt};

pub(crate) fn mutate(
    param: &openapiv3::Parameter,
    the_type: &openapiv3::IntegerType,
) -> ParamMutation {
    let mut mutations = ParamMutation::new_param(&param);
    let (min, max) = the_type.min_max();
    let avg = max / 2; //(min + max) / 2; if max is max of i64 this will overflow

    if param.parameter_data().name == "page" || param.parameter_data().name == "per_page" {
        mutations.push("1", Mutagen::ParamProper);
        return mutations; // These pagination params are resilient to erros, using default if passing wrong params
    } else {
        mutations.push(&avg.to_string(), Mutagen::ParamProper);
    }

    mutations.push(&min.to_string(), Mutagen::Minimum);
    mutations.push(&max.to_string(), Mutagen::Maximum);
    mutations.push(&below_min(min).to_string(), Mutagen::BelowMinimum);
    mutations.push(&over_max(max).to_string(), Mutagen::OverMaximum);
    mutations.push("NotAnInteger", Mutagen::WrongPattern);

    mutations
}

fn below_min(min: i64) -> i64 {
    match min.checked_sub(1) {
        Some(below_min) => below_min,
        None => min, // We are already minimum representable
    }
}

fn over_max(max: i64) -> i64 {
    match max.checked_add(1) {
        Some(over_max) => over_max,
        None => max, // We are already maximum representable
    }
}
