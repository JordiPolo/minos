
use service::QueryParam;
use reqwest::StatusCode;

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
}

pub fn make_query_params<'a>(query_params: Option<&QueryParamMutation<'a>>) -> Option<QueryParam<'a>> {
    match query_params {
        None => None,
        Some(param) => {
            match param {
                QueryParamMutation::Static(the_param) => Some(the_param.clone()),
                QueryParamMutation::Proper => None,
                QueryParamMutation::Wrong => None,
            }
        }
    }
}

pub fn mutations<'a>() -> Vec<Mutation<'a>> {
    vec![
        // Index mutations
        // GET, no params, return 200
        Mutation {
            content_type: "json",
            method: "GET",
            crud_operation: "index",
            query_params: None,
            defined_code: "200",
            expected: StatusCode::OK,
        },

        // GET, unknown params, should be ignored and return 200
        Mutation {
            content_type: "json",
            method: "GET",
            crud_operation: "index",
            query_params: Some(&QueryParamMutation::Static(("trusmis", "mumi"))),
            defined_code: "200",
            expected: StatusCode::OK,        
        },

        // GET, known parameters (if the index have parameters) with proper values, return 200
        Mutation {
            content_type: "json",
            method: "GET",
            crud_operation: "index",
            query_params: Some(&QueryParamMutation::Proper),
            defined_code: "200",
            expected: StatusCode::OK,        
        },

        // GET, known parameters (if it has) with improper values, return 422
        Mutation {
            content_type: "json",
            method: "GET",
            crud_operation: "index",
            query_params: Some(&QueryParamMutation::Wrong),
            defined_code: "422",
            expected: StatusCode::UNPROCESSABLE_ENTITY,        
        },

        // GET, no params, wrong content-type, should return 406
        Mutation {
            content_type: "jason",
            method: "GET",
            crud_operation: "index",
            query_params: None,
            defined_code: "200",
            expected: StatusCode::NOT_ACCEPTABLE,
        },

        // GET, no params, wrong method, should return 405
        Mutation {
            content_type: "json",
            method: "TRACE",
            crud_operation: "index",
            query_params: None,
            defined_code: "200",
            expected: StatusCode::OK,
        },
    ]

}
