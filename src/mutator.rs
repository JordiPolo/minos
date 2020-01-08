use crate::known_param::KnownParamCollection;
use crate::mutation_instructions::MutationInstruction;
use crate::mutation_instructions::ParamMutation;
use crate::operation::Endpoint;
use crate::service::Request;
use crate::request_param::RequestParam;
use chrono::prelude::*;
use openapi_utils::{IntegerTypeExt, ParameterDataExt, ParameterExt, ReferenceOrExt, TypeExt};
use openapiv3::Type;
use std::ops::Range;



pub struct Mutator {
    known_params: KnownParamCollection,
}

impl Mutator {
    pub fn new() -> Self {
        Mutator {
            known_params: KnownParamCollection::new(),
        }
    }

    pub fn request(
        &self,
        endpoint: &Endpoint,
        instructions: &MutationInstruction,
    ) -> Option<Request> {
        let request_path = self.make_path(&endpoint.path_name);

        let mut request_parameters =
            match self.make_query_params(&endpoint.method, &instructions.query_params) {
                None => return None,
                Some(query_params) => query_params
            };

        let mut required_parameters =
            match self.make_query_params(&endpoint.method, &instructions.required_params) {
                None => return None,
                Some(query_params) => query_params
            };

        request_parameters.append(&mut required_parameters);

        let content_type = instructions
            .content_type
            .clone()
            .unwrap_or("application/json".to_string());
        let method = instructions
            .method
            .clone()
            .unwrap_or(endpoint.crud.to_method_name().to_string());

        let request = Request::new()
            .path(request_path)
            .query_params(request_parameters)
            .content_type(content_type)
            .set_method(method);
        Some(request)
    }

    fn make_path(&self, path: &str) -> String {
        match self.known_params.find_by_path(path) {
            None => String::from(path),
            Some(conversion) => str::replace(path, &conversion.pattern, &conversion.value),
        }
    }

    // Returns None when no query params could be created to fulfill this mutation.
    // This happens for instance if we want to create improper parameters
    // But the endpoint does not have any parameters! No request created for this case.
    fn make_query_params(
        &self,
        method: &openapiv3::Operation,
        query_params: &ParamMutation,
    ) -> Option<Vec<RequestParam>> {
        match query_params {
            ParamMutation::None => Some(vec![]),
            ParamMutation::Static(the_param) => {
                Some(vec![RequestParam::new(&the_param.name, &the_param.value)])
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

    fn get_improper_param(&self, method: &openapiv3::Operation) -> Vec<RequestParam> {
        let params_with_types = self.get_only_params_with_types(method);
        // TODO: maybe I dont need result.
        params_with_types
            .into_iter()
            // We can't make improper of pagination params because they get ignored
            // This is an exception, other known but incorrect parameters would fail
            .filter(|x| {
                let name = x.name();
                name != "page" && name != "per_page" && name != "include_count" &&
                // TODO: Improve on this, create improper and expect 404s
                // TODO: If these uuids or searches are required then we should return empty
                // We can't do improper params of uuids or search as most probably we will just get 404, not 422
                !name.ends_with("_uuid") && !name.starts_with("search")
            })
            .map(|param| ImproperParamsBuilder::create_params(&param))
            .collect()
    }

    fn get_proper_param(&self, method: &openapiv3::Operation) -> Vec<RequestParam> {
        let params_with_types = self.get_only_params_with_types(method);

        params_with_types
            .into_iter()
            .filter(|x| {
                let name = x.name();
                let p_type = x.parameter_data().get_type().clone();
                (p_type.is_bool() || p_type.is_integer() || p_type.is_string())
                    && (!name.starts_with("search") || self.known_params.param_known(&name))
            })
            .map(|param| ProperParamsBuilder::create_params(&param, &self.known_params))
            .collect()
    }

    fn get_only_params_with_types(
        &self,
        method: &openapiv3::Operation,
    ) -> Vec<openapiv3::Parameter> {
        method
            .parameters
            .iter()
            .map(|param_ref| param_ref.to_item_ref().clone())
            .collect()
        // TODO: This method should be returning only params with types, we are returning all
        // match params {
        //     None => vec![],
        //     Some(ps) => ps.filter(|x| x.param_type.is_some()).collect(),
        // }
    }
}


fn limits(param: &openapiv3::Parameter) -> Range<i64> {
    match param.parameter_data().get_type() {
        Type::Integer(the_integer) => the_integer.limits(),
        Type::Number(_the_integer) => unimplemented!("Needs support for number"),
        _ => unimplemented!("Figure this out"),
    }
}

struct ProperParamsBuilder;

impl ProperParamsBuilder {
    fn create_params(
        param: &openapiv3::Parameter,
        known_params: &KnownParamCollection,
    ) -> RequestParam {
        //    println!("{:?}", param);
        let name = param.name();
        if known_params.param_known(&name) {
            return RequestParam::new(&name, &known_params.param_value(&name));
        }
        if name == "page" || name == "per_page" || name == "include_count" {
            return ProperParamsBuilder::pagination_param(&param);
        }
        match param.parameter_data().get_type() {
            Type::Boolean {} => ProperParamsBuilder::boolean_request_param(&param),
            Type::Integer(_) => ProperParamsBuilder::integer_request_param(&param),
            Type::String(openapiv3::StringType { format, .. }) => {
                ProperParamsBuilder::string_request_param(&param, &format)
            }
            _ => RequestParam::new(&name, "truething"),
        }
    }

    // Get a valid param, if possible not the default one.
    fn boolean_request_param(param: &openapiv3::Parameter) -> RequestParam {
        RequestParam::new(param.name(), "false")
        // TODO: find default
        // if param.clone().default.unwrap_or(true.into()) == true.into() {
        //     RequestParam::new(&param.name, "false")
        // } else {
        //     RequestParam::new(&param.name, "true")
        // }
    }

    fn integer_request_param(param: &openapiv3::Parameter) -> RequestParam {
        let default: i64 = 1; // TODO: param.clone().default.unwrap_or(1.into()).into();
        let minmax = limits(param);
        let min = minmax.start;
        let max = minmax.end;
        let mut value: i64 = (min + max) / 2;
        if value == default && value < max {
            value += 1;
        }
        RequestParam::new(&param.name(), &format!("{:?}", value))
    }

    // TODO: recover this
    // fn to_string_enum_request_param(param: &openapiv3::Parameter) -> RequestParam {
    //     let enum_values = param
    //         .enum_values
    //         .clone()
    //         .unwrap_or(vec!["string1".to_string()]);
    //     RequestParam::new(
    //         &MinosParam::new(param).get_param_name(),
    //         rand::thread_rng().choose(&enum_values).unwrap(),
    //     )
    // }

    fn string_request_param(
        param: &openapiv3::Parameter,
        format: &openapiv3::VariantOrUnknownOrEmpty<openapiv3::StringFormat>,
    ) -> RequestParam {
        let name = param.parameter_data().name.clone();
        match format {
            openapiv3::VariantOrUnknownOrEmpty::Item(string_format) => match string_format {
                openapiv3::StringFormat::Date => {
                    RequestParam::new(&name, &format!("{:?}", Utc.ymd(2018, 11, 28)))
                }
                openapiv3::StringFormat::DateTime => {
                    let date_time = Utc.ymd(2018, 11, 28).and_hms(12, 0, 9);
                    RequestParam::new(&name, &format!("{:?}", date_time))
                }
                _ => unimplemented!("String format not supported"),
            },
            openapiv3::VariantOrUnknownOrEmpty::Unknown(string) => {
                if string == "uuid" {
                    let uuid = uuid::Uuid::new_v4();
                    RequestParam::new(&name, &format!("{:?}", uuid))
                } else {
                    RequestParam::new(&name, "PLAIN_STRING")
                    // TODO plain string
                    // unimplemented!("No plain string support")
                }
            }
            openapiv3::VariantOrUnknownOrEmpty::Empty => {
                // TODO Better idea here
                RequestParam::new(&name, "PLAIN_STRING")
                //unimplemented!("No plain string support")
            }
        }
        // TODO: This where?
        // ProperParamsBuilder::to_string_enum_request_param(param)
    }

    // Proper pagination params as defined by Github.
    fn pagination_param(param: &openapiv3::Parameter) -> RequestParam {
        let name = param.name();
        if name == "page" {
            RequestParam::new(&name, "1")
        } else if name == "per_page" {
            ProperParamsBuilder::integer_request_param(&param)
        } else {
            // include_count
            RequestParam::new(&name, "true")
        }
    }
}

struct ImproperParamsBuilder;

impl ImproperParamsBuilder {
    fn create_params(param: &openapiv3::Parameter) -> RequestParam {
        let data = param.parameter_data();
        let name = data.name.clone();
        let p_type = data.get_type();

        if p_type.is_bool() {
            RequestParam::new(&name, "-1")
        } else if p_type.is_integer() || p_type.is_number() {
            RequestParam::new(&name, "NotAnIntegerhahahaha")
        } else {
            // if param.maxLength.is_some() {
            //     lenght
            // }
            RequestParam::new(&name, "-1")
        } //string case, not sure how to break it best
    }
}
