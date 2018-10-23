use reqwest::StatusCode;
use service::QueryParam;

pub enum QueryParamMutation<'a> {
    Static(QueryParam<'a>),
    Proper,
    Wrong,
}

pub struct Mutation<'a> {
    pub content_type: &'a str,
    pub method: &'a str,
    pub crud_operation: &'a str,
    pub query_params: Option<&'a QueryParamMutation<'a>>,
    pub defined_code: &'a str,
    pub expected: StatusCode,
    pub explanation: &'a str,
}

pub fn mutations<'a>() -> Vec<Mutation<'a>> {
    vec![
        // Index mutations
        Mutation {
            content_type: "json",
            method: "GET",
            crud_operation: "index",
            query_params: None,
            defined_code: "200",
            expected: StatusCode::OK,
            explanation: "GET with no parameters",
        },
        Mutation {
            content_type: "json",
            method: "GET",
            crud_operation: "index",
            query_params: Some(&QueryParamMutation::Static(("trusmis", "mumi"))),
            defined_code: "200",
            expected: StatusCode::OK,
            explanation: "GET with extra unknown parameters, should be ignored trusmis=mumi",
        },
        Mutation {
            content_type: "json",
            method: "GET",
            crud_operation: "index",
            query_params: Some(&QueryParamMutation::Proper),
            defined_code: "200",
            expected: StatusCode::OK,
            explanation: "GET with extra known and proper parameters",
        },
        Mutation {
            content_type: "json",
            method: "GET",
            crud_operation: "index",
            query_params: Some(&QueryParamMutation::Wrong),
            defined_code: "422",
            expected: StatusCode::UNPROCESSABLE_ENTITY,
            explanation: "GET with extra known but with ilegal parameters",
        },
        Mutation {
            content_type: "jason",
            method: "GET",
            crud_operation: "index",
            query_params: None,
            defined_code: "406", //TODO: this probably does not make sense, as it will never be defined usually
            expected: StatusCode::NOT_ACCEPTABLE,
            explanation: "GET with wrong content-type jason",
        },
        Mutation {
            content_type: "json",
            method: "TRACE",
            crud_operation: "index",
            query_params: None,
            defined_code: "405",
            expected: StatusCode::METHOD_NOT_ALLOWED,
            explanation: "GET with wrong method TRACE",
        },
    ]
}
