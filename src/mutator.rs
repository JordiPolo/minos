use crate::known_param::KnownParamCollection;
use crate::mutation_instructions::MutationInstruction;
use crate::mutation_instructions::{ParamMutation, PathMutation};
use crate::operation::Endpoint;
use crate::request_param::RequestParam;
use crate::service::Request;
use chrono::prelude::*;
use openapi_utils::{IntegerTypeExt, OperationExt, ParameterDataExt, ParameterExt, TypeExt};
use openapiv3::Type;
use std::ops::Range;

pub struct Mutator {
    known_params: KnownParamCollection,
}

impl Mutator {
    pub fn new(conversions: &str) -> Self {
        Mutator {
            known_params: KnownParamCollection::new(conversions),
        }
    }

    pub fn request(
        &self,
        endpoint: &Endpoint,
        instructions: &MutationInstruction,
    ) -> Option<Request> {
        let request_path = self.make_path(&endpoint.path_name, &instructions)?;
        let request_parameters = self.make_query_params(&endpoint, &instructions)?;

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

    fn make_path(&self, path: &str, instructions: &MutationInstruction) -> Option<String> {
        if !path.contains('}') {
            Some(String::from(path))
        } else {
            let conversion = self.known_params.find_by_path(path)?;
            match instructions.path_params {
                PathMutation::Proper => {
                    Some(str::replace(path, &conversion.pattern, &conversion.value))
                }
                PathMutation::Random => {
                    Some(str::replace(path, &conversion.pattern, "wrongPathItemHere"))
                }
            }
        }
    }

    fn make_query_params(
        &self,
        endpoint: &Endpoint,
        instructions: &MutationInstruction,
    ) -> Option<Vec<RequestParam>> {
        // TODO: A hack to special case this but this would otherwise produce a mutation which will not fail
        // even when the instructions say it would
        if endpoint.method.required_parameters().is_empty()
            && instructions.required_params == crate::mutation_instructions::ParamMutation::None
        {
            return None;
        }

        let mut request_parameters = match self.mutate_query_params(
            &endpoint.method.required_parameters(),
            &instructions.required_params,
        ) {
            None => return None,
            Some(query_params) => query_params,
        };

        let mut required_parameters = match self.mutate_query_params(
            &endpoint.method.optional_parameters(),
            &instructions.query_params,
        ) {
            None => return None,
            Some(query_params) => query_params,
        };

        request_parameters.append(&mut required_parameters);
        Some(request_parameters)
    }

    // Returns None when no query params could be created to fulfill this mutation.
    // This happens for instance if we want to create improper parameters
    // But the endpoint does not have any parameters! No request created for this case.
    fn mutate_query_params(
        &self,
        params: &Vec<&openapiv3::Parameter>,
        query_params: &ParamMutation,
    ) -> Option<Vec<RequestParam>> {
        // Can't create any mutation if there is no definition

        match query_params {
            ParamMutation::None => Some(vec![]),
            ParamMutation::Static(the_param) => {
                Some(vec![RequestParam::new(&the_param.name, &the_param.value)])
            }
            ParamMutation::Proper => self.get_proper_param(params),
            // TODO: properly find wrong parameter here
            ParamMutation::Wrong => {
                let improper_params = self.get_improper_param(params);
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

    fn get_improper_param(&self, params: &Vec<&openapiv3::Parameter>) -> Vec<RequestParam> {
        let params_with_types = self.get_only_params_with_types(params.clone());

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

    fn get_proper_param(&self, params: &Vec<&openapiv3::Parameter>) -> Option<Vec<RequestParam>> {
        let params_with_types = self.get_only_params_with_types(params.clone());

        params_with_types
            .into_iter()
            .filter(|x| {
                let name = x.name();
                let p_type = x.parameter_data().get_type().clone();
                (p_type.is_bool() || p_type.is_integer() || p_type.is_string())
                    && (!name.starts_with("search") || self.known_params.param_known(&name))
            })
            .map(|param| ProperParamsBuilder::create_params(&param, &self.known_params))
            .collect() // This will transform Vec<Option<X>> to Option<Vec<X>>
    }

    fn get_only_params_with_types<'a>(
        &self,
        params: Vec<&'a openapiv3::Parameter>,
    ) -> Vec<&'a openapiv3::Parameter> {
        params
            .into_iter()
            .filter(|&p| p.parameter_data().is_type_defined())
            .collect()
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
    ) -> Option<RequestParam> {
        let name = param.name();
        if known_params.param_known(&name) {
            return Some(RequestParam::new(&name, &known_params.param_value(&name)));
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
            _ => Some(RequestParam::new(&name, "truething")),
        }
    }

    // Get a valid param, if possible not the default one.
    fn boolean_request_param(param: &openapiv3::Parameter) -> Option<RequestParam> {
        Some(RequestParam::new(param.name(), "false"))
        // TODO: find default
        // if param.clone().default.unwrap_or(true.into()) == true.into() {
        //     RequestParam::new(&param.name, "false")
        // } else {
        //     RequestParam::new(&param.name, "true")
        // }
    }

    fn integer_request_param(param: &openapiv3::Parameter) -> Option<RequestParam> {
        let default: i64 = 1; // TODO: param.clone().default.unwrap_or(1.into()).into();
        let minmax = limits(param);
        let min = minmax.start;
        let max = minmax.end;
        let mut value: i64 = (min + max) / 2;
        if value == default && value < max {
            value += 1;
        }
        Some(RequestParam::new(&param.name(), &format!("{:?}", value)))
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
    ) -> Option<RequestParam> {
        let name = param.parameter_data().name.clone();
        match format {
            openapiv3::VariantOrUnknownOrEmpty::Item(string_format) => match string_format {
                openapiv3::StringFormat::Date => Some(RequestParam::new(
                    &name,
                    &format!("{:?}", Utc.ymd(2019, 11, 28)),
                )),
                openapiv3::StringFormat::DateTime => {
                    let date_time = Utc.ymd(2019, 11, 28).and_hms(12, 0, 9);
                    Some(RequestParam::new(&name, &format!("{:?}", date_time)))
                }
                _ => unimplemented!("String format not supported"),
            },
            openapiv3::VariantOrUnknownOrEmpty::Unknown(string) => {
                if string == "uuid" {
                    // We can't do a random uuid, as it will fail. None says we did not create a valid param
                    None
                // let uuid = uuid::Uuid::new_v4();
                // RequestParam::new(&name, &format!("{:?}", uuid))
                } else {
                    Some(RequestParam::new(&name, "PLAIN_STRING"))
                    // TODO plain string
                    // unimplemented!("No plain string support")
                }
            }
            openapiv3::VariantOrUnknownOrEmpty::Empty => {
                // TODO Better idea here
                Some(RequestParam::new(&name, "PLAIN_STRING"))
            }
        }
        // TODO: This where?
        // ProperParamsBuilder::to_string_enum_request_param(param)
    }

    // Proper pagination params as defined by Github.
    fn pagination_param(param: &openapiv3::Parameter) -> Option<RequestParam> {
        let name = param.name();
        if name == "page" {
            Some(RequestParam::new(&name, "1"))
        } else if name == "per_page" {
            ProperParamsBuilder::integer_request_param(&param)
        } else {
            // include_count
            Some(RequestParam::new(&name, "true"))
        }
    }
}

struct ImproperParamsBuilder;

impl ImproperParamsBuilder {
    fn create_params(param: &openapiv3::Parameter) -> RequestParam {
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
}
