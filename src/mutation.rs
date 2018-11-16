use operation::CRUD;
use reqwest::StatusCode;
use service::QueryParam;

pub enum ParamMutation {
    Static(QueryParam),
    Proper,
    Wrong,
    None,
    //    Empty, what would happen in this case?
}

impl ParamMutation {
    fn static_values(name: &str, value: &str) -> Self {
        ParamMutation::Static((name.to_string(), value.to_string()))
    }
}

pub struct Mutation<'a> {
    pub content_type: &'a str,
    pub method: &'a str,
    pub crud_operation: CRUD,
    pub required_params: ParamMutation, // Required parameters in path or query string
    pub query_params: ParamMutation,    // Optional query parameters
    pub expected: StatusCode,
    pub explanation: &'a str,
}

pub fn mutations_for_crud<'a>(crud: CRUD) -> Vec<Mutation<'a>> {
    mutations()
        .into_iter()
        .filter(|mutation| mutation.crud_operation == crud)
        .collect()
}

impl<'a> Mutation<'a> {
    fn new(explanation: &'a str) -> Self {
        Mutation {
            content_type: "application/json",
            method: "GET",
            crud_operation: CRUD::Index,
            required_params: ParamMutation::Proper,
            query_params: ParamMutation::None,
            expected: StatusCode::OK,
            explanation,
        }
    }

    fn content_type(mut self, content_type: &'a str) -> Self {
        self.content_type = content_type;
        self
    }

    fn expected(mut self, expected: StatusCode) -> Self {
        self.expected = expected;
        self
    }

    fn query_params(mut self, query_params: ParamMutation) -> Self {
        self.query_params = query_params;
        self
    }

    fn method(mut self, method: &'a str) -> Self {
        self.method = method;
        self
    }
}

pub fn mutations<'a>() -> Vec<Mutation<'a>> {
    vec![
        Mutation::new("GET with no optional parameters")
            .query_params(ParamMutation::None)
            .expected(StatusCode::OK),
        Mutation::new("GET with extra known optional and proper parameters")
            .query_params(ParamMutation::Proper)
            .expected(StatusCode::OK),
        Mutation::new("GET with extra unknown parameters <trusmis=mumi>")
            .query_params(ParamMutation::static_values("trusmis", "mumi"))
            .expected(StatusCode::OK),
        Mutation::new("GET with extra known optional but with improper parameters")
            .query_params(ParamMutation::Wrong)
            .expected(StatusCode::UNPROCESSABLE_ENTITY),
        Mutation::new("GET with wrong content-type <jason>")
            .content_type("minosTest/jason")
            .expected(StatusCode::NOT_ACCEPTABLE),
        Mutation::new("Request with wrong method <TRACE>")
            .method("TRACE")
            .expected(StatusCode::METHOD_NOT_ALLOWED),
    ]
}
