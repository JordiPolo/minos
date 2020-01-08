// This is the Spec Request param information and helper methods.
#[derive(Debug, PartialEq)]
pub struct RequestParam {
    pub name: String,
    pub value: String,
}

impl RequestParam {
    pub fn new(name: &str, value: &str) -> Self {
        RequestParam {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}
