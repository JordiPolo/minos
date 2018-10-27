use reqwest::StatusCode;
use service::QueryParam;
use operation::CRUD;

pub enum QueryParamMutation {
    Static(QueryParam),
    Proper,
    Wrong,
//    Empty, what would happen in this case?
}

pub struct Mutation<'a> {
    pub content_type: &'a str,
    pub method: &'a str,
    pub crud_operation: CRUD,
    pub query_params: Option<QueryParamMutation>,
    pub defined_code: &'a str,
    pub expected: StatusCode,
    pub explanation: &'a str,
}

impl<'a> Mutation<'a> {

    pub fn is_application_defined_code(&self) -> bool {
        let appplication_codes = vec!["200", "400", "403", "404", "409", "422", "423"];
        appplication_codes.iter().find(|&&code| code == self.defined_code).is_some()
    }
    // pub fn is_middleware_code(&self) -> bool {
    //     let appplication_codes = vec!["401", "500"];
    //     appplication_codes.iter().find(|&&code| code == self.defined_code).is_some()
    // }
    // pub fn is_framework_code(&self) -> bool {
    //     !(self.is_application_defined_code() || self.is_application_defined_code())
    // }
}

pub fn mutations_for_crud<'a>(crud: CRUD) -> Vec<Mutation<'a>> {
    mutations().into_iter().filter(|mutation| mutation.crud_operation == crud).collect()
}


pub fn mutations<'a>() -> Vec<Mutation<'a>> {
    vec![
        // Index mutations
        Mutation {
            content_type: "json",
            method: "GET",
            crud_operation: CRUD::Index,
            query_params: None,
            defined_code: "200",
            expected: StatusCode::OK,
            explanation: "GET with no parameters",
        },
        Mutation {
            content_type: "json",
            method: "GET",
            crud_operation: CRUD::Index,
            query_params: Some(QueryParamMutation::Static(("trusmis".to_string(), "mumi".to_string()))),
            defined_code: "200",
            expected: StatusCode::OK,
            explanation: "GET with extra unknown parameters, should be ignored trusmis=mumi",
        },
        Mutation {
            content_type: "json",
            method: "GET",
            crud_operation: CRUD::Index,
            query_params: Some(QueryParamMutation::Proper),
            defined_code: "200",
            expected: StatusCode::OK,
            explanation: "GET with extra known and proper parameters",
        },
        Mutation {
            content_type: "json",
            method: "GET",
            crud_operation: CRUD::Index,
            query_params: Some(QueryParamMutation::Wrong),
            defined_code: "422",
            expected: StatusCode::UNPROCESSABLE_ENTITY,
            explanation: "GET with extra known but with ilegal parameters",
        },
        Mutation {
            content_type: "jason",
            method: "GET",
            crud_operation: CRUD::Index,
            query_params: None,
            defined_code: "406", //TODO: this probably does not make sense, as it will never be defined usually
            expected: StatusCode::NOT_ACCEPTABLE,
            explanation: "GET with wrong content-type jason",
        },
        Mutation {
            content_type: "json",
            method: "TRACE",
            crud_operation: CRUD::Index,
            query_params: None,
            defined_code: "405",
            expected: StatusCode::METHOD_NOT_ALLOWED,
            explanation: "GET with wrong method TRACE",
        },
    ]
}
