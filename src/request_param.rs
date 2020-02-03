// This is the Spec Request param information and helper methods.
#[derive(Debug, PartialEq, Clone)]
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
    pub fn new2(name: &str, value: Option<&str>) -> Self {
        RequestParam {
            name: name.to_string(),
            value: value.map(|s| s.to_string()),
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

