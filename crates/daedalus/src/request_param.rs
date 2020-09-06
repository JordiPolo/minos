// This is the Spec Request param information and helper methods.
// We want to be able to render the name of the parameter that we do send
// On value: None the value is not serialized.
#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord)]
pub struct RequestParam {
    pub name: String,
    pub value: Option<String>,
}

impl RequestParam {
    pub fn new(name: &str, value: &str) -> Self {
        RequestParam {
            name: name.to_string(),
            value: Some(value.to_string()),
        }
    }
    pub fn new2(name: &str, value: Option<String>) -> Self {
        RequestParam {
            name: name.to_string(),
            value,
        }
    }
}
// use crate::mutation::param_mutation::ParamMutation;

// impl From<ParamMutation> for Vec<RequestParam> {
//     fn from(mutation: ParamMutation) -> Self {
//             mutation.variations
//                 .iter()
//                 .map(|variation| {
//                     (
//                         RequestParam::new(&mutation.name, &variation.param),
//                         variation.mutagen.clone(),
//                     )
//                 })
//                 .collect()
//         }
//     }
