use crate::request_param::RequestParam;
use openapi_utils::{ParameterDataExt, ParameterExt};
use openapiv3::Type;

pub fn create_params(param: &openapiv3::Parameter) -> RequestParam {
    let data = param.parameter_data();

    match data.get_type() {
        Type::Boolean { .. } => RequestParam::new(&data.name, "-1"),
        Type::Integer { .. } | Type::Number { .. } => {
            RequestParam::new(&data.name, "NotAnIntegerhahahaha")
        }
        Type::String { .. } => RequestParam::new(&data.name, "-1"), // TODO check format and make something wrong
        Type::Array { .. } => RequestParam::new(&data.name, "notAnArray"),
        Type::Object { .. } => RequestParam::new(&data.name, "notAnObject"),
    }
}
