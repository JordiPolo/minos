use rand::Rng;

use mutation::ParamMutation;
use openapi;
use spec::Spec;

//// From here is all about reading known params from files, maybe move somewhere else
#[derive(Debug)]
pub struct KnownParam {
    pattern: String,
    value: String,
    context: String,
}

impl KnownParam {
    pub fn new(data: &[&str]) -> Self {
        KnownParam {
            pattern: data[1].to_string(),
            value: data[2].to_string(),
            context: data[0].to_string(),
        }
    }

    fn path_matches(&self, full_path: &str) -> bool {
        self.context == "path" && full_path.contains(&self.pattern)
    }

    fn query_matches(&self, query_param_name: &str) -> bool {
        self.context == "query" && query_param_name == self.pattern
    }
}

// TODO: Super inefficiently we are reading the file 1000 times
fn param_known(name: &str) -> bool {
    read_known_params()
        .into_iter()
        .any(|param| param.query_matches(name))
}

fn param_value(name: &str) -> String {
    read_known_params()
        .into_iter()
        .find(|param| param.query_matches(name))
        .unwrap()
        .value
        .to_string()
}

fn read_known_params() -> Vec<KnownParam> {
    let mut result = vec![];
    match std::fs::read_to_string("conversions.minos") {
        Err(_) => vec![],
        Ok(contents) => {
            for line in contents.split('\n') {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() == 3 {
                    result.push(KnownParam::new(&parts))
                }
            }
            println!("{:?}", result);
            result
        }
    }
}
///// Till here

pub fn make_path(path: &str) -> String {
    match read_known_params()
        .iter()
        .find(|&known_param| known_param.path_matches(path))
    {
        None => String::from(path),
        Some(conversion) => str::replace(path, &conversion.pattern, &conversion.value),
    }
}

// This is the Spec Request param information and helper methods.
#[derive(Clone)]
pub struct RequestParam {
    pub name: String,
    pub value: String,
}

pub fn make_query_params(
    spec: &Spec,
    method: &openapi::v2::Operation,
    query_params: &ParamMutation,
) -> Option<Vec<RequestParam>> {
    match query_params {
        ParamMutation::None => Some(vec![]),
        ParamMutation::Static(the_param) => Some(vec![RequestParam {
            name: the_param.0.to_string(),
            value: the_param.1.to_string(),
        }]),
        ParamMutation::Proper => Some(get_proper_param(&spec, method)),
        // TODO: properly find wrong parameter here
        ParamMutation::Wrong => {
            let improper_params = get_improper_param(&spec, method);
            // If we could not find improper parameters we return None to skip this test
            // TODO. This is not a very good way of communicating the intent
            if improper_params.is_empty() {
                None
            } else {
                Some(improper_params)
            }
        }
        // QueryParamMutation::Empty => {
        //     let proper_params = request_params::get_proper_param(&spec, method);
        //     let result = proper_params.into_iter().map(|mut param| {param.value = "".to_string(); param }).collect();
        //     Some(result)
        // }
    }
}

fn get_improper_param(spec: &Spec, method: &openapi::v2::Operation) -> Vec<RequestParam> {
    let params_with_types = get_only_params_with_types(spec, method);

    // TODO: maybe I dont need result.
    params_with_types
        .into_iter()
        // We can't make improper of pagination params because they get ignored
        // This is an exception, other known but incorrect parameters would fail
        .filter(|x| {
            let name = x.clone().name;
            name != "page" && name != "per_page" && name != "include_count" &&
            // TODO: Improve on this, create improper and expect 404s
            // TODO: If these uuids or searches are required then we should return empty
            // We can't do improper params of uuids or search as most probably we will just get 404, not 422
            !name.ends_with("_uuid") && !name.starts_with("search")
        }).map(|param| {
            let p_type = param.clone().param_type.unwrap();
            let name = param.clone().name;
            let result;
            if p_type == "boolean" {
                result = RequestParam {
                    name,
                    value: "-1".to_string(),
                };
            } else if p_type == "integer" || p_type == "numeric" {
                result = RequestParam {
                    name,
                    value: "NotAnIntegerhahahaha".to_string(),
                };
            } else {
                // if param.maxLength.is_some() {
                //     lenght
                // }
                result = RequestParam {
                    name,
                    value: "-1".to_string(),
                };
            } //string case, not sure how to break it best
            result
        }).collect()
}

fn get_proper_param(spec: &Spec, method: &openapi::v2::Operation) -> Vec<RequestParam> {
    let params_with_types = get_only_params_with_types(spec, method);

    let request_params = params_with_types
        .into_iter()
        .filter(|x| {
            let name = x.clone().name;
            let p_type = x.clone().param_type.unwrap();
            (p_type == "boolean" || p_type == "integer" || p_type == "string")
                && ((!name.ends_with("_uuid") && !name.starts_with("search")) || param_known(&name))
            //        && (name != "page" && name != "per_page" && name != "include_count")
        }).map(|param| to_request_param(&param));

    request_params.collect()
}

fn get_only_params_with_types(
    spec: &Spec,
    method: &openapi::v2::Operation,
) -> Vec<openapi::v2::Parameter> {
    let params = match &method.parameters {
        Some(param) => Some(param.into_iter().map(|p| spec.resolve_parameter_ref(&p))),
        None => None,
    };

    match params {
        None => vec![],
        Some(ps) => ps.filter(|x| x.param_type.is_some()).collect(),
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
        value += 1;
    }
    RequestParam::new(&param.name, &format!("{:?}", value))
}

use chrono::prelude::*;

fn to_string_enum_request_param(param: &openapi::v2::Parameter) -> RequestParam {
    let enum_values = param
        .enum_values
        .clone()
        .unwrap_or(vec!["string1".to_string()]);
    RequestParam::new(
        &param.name,
        rand::thread_rng().choose(&enum_values).unwrap(),
    )
}

fn to_string_request_param(param: &openapi::v2::Parameter) -> RequestParam {
    if param.format.is_some() {
        let format = param.format.clone().unwrap();
        // TODO: PRobably we need to do much better than just random UUIDs
        if format == "uuid" {
            let uuid = uuid::Uuid::new_v4();
            RequestParam::new(&param.name, &format!("{:?}", uuid))
        } else if format == "date-time" {
            let date_time = Utc.ymd(2018, 11, 28).and_hms(12, 0, 9);
            RequestParam::new(&param.name, &format!("{:?}", date_time))
        } else {
            //if format == "date" {
            let date = Utc.ymd(2018, 11, 28);
            RequestParam::new(&param.name, &format!("{:?}", date))
        }
    // Need to do uuids but how useful is this without being able to find uuids in the system?
    } else {
        to_string_enum_request_param(param)
    }
}

fn to_pagination_param(param: &openapi::v2::Parameter) -> RequestParam {
    if param.name == "page" {
        RequestParam::new(&param.name, "1")
    } else if param.name == "per_page" {
        to_integer_request_param(&param)
    } else {
        // include_count
        RequestParam::new(&param.name, "true")
    }
}

fn to_request_param(param: &openapi::v2::Parameter) -> RequestParam {
    let p = param.clone();
    let p_type = p.param_type.unwrap();
    if param_known(&p.name) {
        return RequestParam::new(&p.name, &param_value(&p.name));
    }
    if p.name == "page" || p.name == "per_page" || p.name == "include_count" {
        return to_pagination_param(&param);
    }
    if p_type == "boolean" {
        to_boolean_request_param(&param)
    } else if p_type == "integer" {
        to_integer_request_param(&param)
    } else if p_type == "string" {
        to_string_request_param(&param)
    } else {
        RequestParam::new(&param.name, "truething")
    }
}
