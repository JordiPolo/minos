use crate::known_param::KnownParamCollection;
use crate::mutation::instructions::MutationInstruction;
use crate::mutation::instructions::{ParamMutation, PathMutation};
use crate::operation::Endpoint;
use crate::request_param::RequestParam;
use crate::service::Request;
use openapi_utils::{OperationExt, ParameterDataExt, ParameterExt, TypeExt};

mod improper_params;
pub mod instructions;
mod proper_params;

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
        if path.contains('}') {
            let conversion = self.known_params.find_by_path(path)?;
            match instructions.path_params {
                PathMutation::Proper => {
                    Some(str::replace(path, &conversion.pattern, &conversion.value))
                }
                PathMutation::Random => {
                    Some(str::replace(path, &conversion.pattern, "wrongPathItemHere"))
                }
            }
        } else {
            match instructions.path_params {
                PathMutation::Proper => Some(String::from(path)),
                PathMutation::Random => {
                    None // We can't make random something that's is not there
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
            && instructions.required_params == crate::mutation::instructions::ParamMutation::None
        {
            return None;
        }

        let mut request_parameters = self.mutate_query_params(
            &endpoint.method.required_parameters(),
            &instructions.required_params,
        )?;

        let mut required_parameters = self.mutate_query_params(
            &endpoint.method.optional_parameters(),
            &instructions.query_params,
        )?;

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
            } // ParamMutation::Empty => {
              //       let proper_params = self.get_proper_param(params)?;
              //       let result = proper_params.into_iter().map(|mut param| {param.value = "".to_string(); param }).collect();
              //       Some(result)
              //   }
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
                !name.ends_with("_uuid") && !name.starts_with("search") &&
                x.location_string() == "query"
            })
            .map(|param| improper_params::create_params(&param))
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
                    && x.location_string() == "query"
            })
            .map(|param| proper_params::create_params(&param, &self.known_params))
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
