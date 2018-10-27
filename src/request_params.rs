use rand::Rng;

use openapi;
use spec::Spec;

// This is the Spec Request param information and helper methods.
#[derive(Clone)]
pub struct RequestParam {
    pub name: String,
    pub value: String,
}

pub fn get_improper_param(spec: &Spec, method: &openapi::v2::Operation) -> Vec<RequestParam> {
    let params_with_types = get_only_params_with_types(spec, method);

    // TODO: maybe I dont need result.
    params_with_types.into_iter().map(|param|
    {
        let p_type = param.clone().param_type.unwrap();
        let name = param.clone().name;
        let result;
        if p_type == "boolean" {result = RequestParam{name, value: "-1".to_string()};}
        else if p_type == "integer"  {result = RequestParam{name, value: "NotAnIntegerhahahaha".to_string() };}
        else  {result = RequestParam{name, value: "-1".to_string() };} //string case, not sure how to break it best
        result
    }).collect()
}


pub fn get_proper_param(spec: &Spec, method: &openapi::v2::Operation) -> Vec<RequestParam> {
    let params_with_types = get_only_params_with_types(spec, method);

    let request_params = params_with_types.into_iter()
        .filter(|x| {
            let name = x.clone().name;
            let p_type = x.clone().param_type.unwrap();
            (p_type == "boolean" || p_type == "integer" || p_type == "string")
                && (name != "page" && name != "per_page" && name != "include_count")
        }).map(|param| to_request_param(&param));

    request_params.collect()
}


fn get_only_params_with_types(spec: &Spec, method: &openapi::v2::Operation) -> Vec<openapi::v2::Parameter> {
    let params = match &method.parameters {
        Some(param) => Some(param.into_iter().map(|p| resolve_parameter_ref(&spec, &p))),
        None => None,
    };

    match params {
        None => vec![],
        Some(ps) => {
        ps
            .into_iter()
            .filter(|x| x.param_type.is_some())
            .collect()
        }
    }
}

impl RequestParam {
    fn new(name: &str, value: &str) -> Self {
        RequestParam {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}

// TODO: copied in main.rs
pub fn resolve_parameter_ref(
    spec: &Spec,
    param_or_ref: &openapi::v2::ParameterOrRef,
) -> openapi::v2::Parameter {
    match param_or_ref.clone() {
        openapi::v2::ParameterOrRef::Parameter {
            name,
            location,
            required,
            schema,
            unique_items,
            param_type,
            format,
            description,
            minimum,
            maximum,
            default,
            enum_values,
        } => openapi::v2::Parameter {
            name,
            location,
            required,
            schema,
            unique_items,
            param_type,
            format,
            description,
            minimum,
            maximum,
            default,
            enum_values,
        },
        openapi::v2::ParameterOrRef::Ref { ref_path } => spec.resolve_parameter(&ref_path),
    }
}

// Get a valid param, if possible not the default one.
fn to_boolean_request_param(param: &openapi::v2::Parameter) -> RequestParam {
    if param.clone().default.unwrap_or(true.into()) == true.into() {
        RequestParam::new(&param.name, "false")
    } else {
        RequestParam::new(&param.name, "true")
    }
}

fn to_integer_request_param(param: &openapi::v2::Parameter) -> RequestParam {
    let default: i32 = param.clone().default.unwrap_or(1.into()).into();
    let min = param.minimum.unwrap_or(1);
    let max = param.maximum.unwrap_or(100);
    let mut value: i32 = (min + max) / 2;
    if value == default && value < max {
        value = value + 1;
    }
    RequestParam::new(&param.name, &format!("{:?}", value))
}

fn to_string_enum_request_param(param: &openapi::v2::Parameter) -> RequestParam {
    let enum_values = param.clone().enum_values.unwrap_or(vec!["string1".to_string()]);
    RequestParam::new(&param.name, rand::thread_rng().choose(&enum_values).unwrap())
}

fn to_request_param(param: &openapi::v2::Parameter) -> RequestParam {
    let p = param.clone();
    let p_type = p.param_type.unwrap();
    if p_type == "boolean" {
        to_boolean_request_param(&param)
    } else if p_type == "integer" {
        to_integer_request_param(&param)
    } else if p_type == "string" {
        to_string_enum_request_param(&param)
    } else {
        RequestParam::new(&param.name, "truething")
    }
}
