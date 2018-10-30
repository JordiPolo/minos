use operation::CRUD;
use reqwest::StatusCode;
use service::QueryParam;

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
    pub expected: StatusCode,
    pub explanation: &'a str,
}

impl<'a> Mutation<'a> {
    pub fn is_application_defined_code(&self) -> bool {
        let appplication_codes = vec!["200", "400", "403", "404", "409", "422", "423"];
        appplication_codes
            .iter()
            .any(|&code| code == self.defined_code)
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
    mutations()
        .into_iter()
        .filter(|mutation| mutation.crud_operation == crud)
        .collect()
}

pub fn mutations<'a>() -> Vec<Mutation<'a>> {
    vec![
        // Index mutations
        Mutation {
            content_type: "application/json",
            method: "GET",
            crud_operation: CRUD::Index,
            query_params: None,
            expected: StatusCode::OK,
            explanation: "GET index with no parameters",
        },
        Mutation {
            content_type: "application/json",
            method: "GET",
            crud_operation: CRUD::Index,
            query_params: Some(QueryParamMutation::Static((
                "trusmis".to_string(),
                "mumi".to_string(),
            ))),
            expected: StatusCode::OK,
            explanation: "GET with extra unknown parameters <trusmis=mumi>",
        },
        Mutation {
            content_type: "application/json",
            method: "GET",
            crud_operation: CRUD::Index,
            query_params: Some(QueryParamMutation::Proper),
            expected: StatusCode::OK,
            explanation: "GET with extra known and proper parameters",
        },
        Mutation {
            content_type: "application/json",
            method: "GET",
            crud_operation: CRUD::Index,
            query_params: Some(QueryParamMutation::Wrong),
            expected: StatusCode::UNPROCESSABLE_ENTITY,
            explanation: "GET with extra known but with improper parameters",
        },
        Mutation {
            content_type: "minosTest/jason",
            method: "GET",
            crud_operation: CRUD::Index,
            query_params: None,
            expected: StatusCode::NOT_ACCEPTABLE, //TODO: this probably does not make sense, as it will never be defined usually
            explanation: "GET with wrong content-type <jason>",
        },
        Mutation {
            content_type: "application/json",
            method: "TRACE",
            crud_operation: CRUD::Index,
            query_params: None,
            expected: StatusCode::METHOD_NOT_ALLOWED,
            explanation: "GET with wrong method <TRACE>",
        },
    ]
}
