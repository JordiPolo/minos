use chrono::prelude::*;
use openapi;
use rand::Rng;

use crate::known_param::KnownParamCollection;
use crate::mutation_instructions::ParamMutation;

pub struct Mutator {
    known_params: KnownParamCollection,
}

// This is the Spec Request param information and helper methods.
#[derive(Clone)]
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




struct ProperParamsBuilder;

impl ProperParamsBuilder {
    pub fn create_params(
        param: &openapi::v2::Parameter,
        known_params: &KnownParamCollection,
    ) -> RequestParam {
        let p = param.clone();
        println!("{:?}", param);
        let p_type = p.param_type.unwrap();
        if known_params.param_known(&p.name) {
            return RequestParam::new(&p.name, &known_params.param_value(&p.name));
        }
        if p.name == "page" || p.name == "per_page" || p.name == "include_count" {
            return ProperParamsBuilder::to_pagination_param(&param);
        }
        if p_type == "boolean" {
            ProperParamsBuilder::to_boolean_request_param(&param)
        } else if p_type == "integer" {
            ProperParamsBuilder::to_integer_request_param(&param)
        } else if p_type == "string" {
            ProperParamsBuilder::to_string_request_param(&param)
        } else {
            RequestParam::new(&param.name, "truething")
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
            ProperParamsBuilder::to_string_enum_request_param(param)
        }
    }

    // Proper pagination params as defined by Github.
    fn to_pagination_param(param: &openapi::v2::Parameter) -> RequestParam {
        if param.name == "page" {
            RequestParam::new(&param.name, "1")
        } else if param.name == "per_page" {
            ProperParamsBuilder::to_integer_request_param(&param)
        } else {
            // include_count
            RequestParam::new(&param.name, "true")
        }
    }
}

struct ImproperParamsBuilder;

impl ImproperParamsBuilder {
    pub fn create_params(
        param: &openapi::v2::Parameter,
    ) -> RequestParam {
        let p_type = param.clone().param_type.unwrap();
        let name = param.clone().name;
        let result;
        if p_type == "boolean" {
            result = RequestParam::new(&name, "-1");
        } else if p_type == "integer" || p_type == "numeric" {
            result = RequestParam::new(&name, "NotAnIntegerhahahaha");
        } else {
            // if param.maxLength.is_some() {
            //     lenght
            // }
            result = RequestParam::new(&name, "-1");
        } //string case, not sure how to break it best
        result
    }
}




impl Mutator {
    pub fn new() -> Self {
        Mutator {
            known_params: KnownParamCollection::new(),
        }
    }

    pub fn make_path(&self, path: &str) -> String {
        match self.known_params.find_by_path(path) {
            None => String::from(path),
            Some(conversion) => str::replace(path, &conversion.pattern, &conversion.value),
        }
    }

    pub fn make_query_params(
        &self,
        method: &openapi::v2::Operation,
        query_params: &ParamMutation,
    ) -> Option<Vec<RequestParam>> {
        match query_params {
            ParamMutation::None => Some(vec![]),
            ParamMutation::Static(the_param) => {
                Some(vec![RequestParam::new(&the_param.0, &the_param.1)])
            }
            ParamMutation::Proper => Some(self.get_proper_param(method)),
            // TODO: properly find wrong parameter here
            ParamMutation::Wrong => {
                let improper_params = self.get_improper_param(method);
                // If we could not find improper parameters we return None to skip this test
                // TODO. This is not a very good way of communicating the intent
                if improper_params.is_empty() {
                    None
                } else {
                    Some(improper_params)
                }
            } // QueryParamMutation::Empty => {
              //     let proper_params = request_params::get_proper_param(&spec, method);
              //     let result = proper_params.into_iter().map(|mut param| {param.value = "".to_string(); param }).collect();
              //     Some(result)
              // }
        }
    }

    fn get_improper_param(&self, method: &openapi::v2::Operation) -> Vec<RequestParam> {
        let params_with_types = self.get_only_params_with_types(method);

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
            }).map(|param| ImproperParamsBuilder::create_params(&param)).collect()
    }

    fn get_proper_param(&self, method: &openapi::v2::Operation) -> Vec<RequestParam> {
        let params_with_types = self.get_only_params_with_types(method);

        params_with_types
            .into_iter()
            .filter(|x| {
                let name = x.clone().name;
                let p_type = x.clone().param_type.unwrap();
                (p_type == "boolean" || p_type == "integer" || p_type == "string")
                    && (!name.starts_with("search") || self.known_params.param_known(&name))
            }).map(|param| ProperParamsBuilder::create_params(&param, &self.known_params)).collect()

    }

   fn parameter_ref_to_parameter(
    param_or_ref: &openapi::v2::ParameterOrRef,
    ) -> openapi::v2::Parameter {
        match param_or_ref.clone() {
            openapi::v2::ParameterOrRef::Parameter(parameter) => parameter,
            openapi::v2::ParameterOrRef::Ref { .. } => unreachable!(),
        }
    }

    fn get_only_params_with_types(
        &self,
        method: &openapi::v2::Operation,
    ) -> Vec<openapi::v2::Parameter> {
        let params = match &method.parameters {
            Some(param) => Some(
                param
                    .into_iter()
                    .map(|p| Mutator::parameter_ref_to_parameter(&p)),
            ),
            None => None,
        };

        match params {
            None => vec![],
            Some(ps) => ps.filter(|x| x.param_type.is_some()).collect(),
        }
    }
}
