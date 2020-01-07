use crate::operation::CRUD;
use crate::service::QueryParam;
use reqwest::StatusCode;

#[derive(Debug)]
pub struct MutationInstruction<'a> {
    pub content_type: &'a str,
    pub method: &'a str,
    pub crud_operation: CRUD,
    pub required_params: ParamMutation, // Required parameters in path or query string
    pub query_params: ParamMutation,    // Optional query parameters
    pub expected: StatusCode,
    pub explanation: &'a str,
}

#[derive(Debug)]
pub enum ParamMutation {
    Static(QueryParam),
    Proper,
    Wrong,
    None,
    //    Empty, what would happen in this case?
}

pub fn instructions_for_operation<'a>(crud: CRUD) -> Vec<MutationInstruction<'a>> {
    mutations()
        .into_iter()
        .filter(|mutation| mutation.crud_operation == crud)
        .collect()
}

impl ParamMutation {
    fn static_values(name: &str, value: &str) -> Self {
        ParamMutation::Static((name.to_string(), value.to_string()))
    }
}

impl<'a> MutationInstruction<'a> {
    fn new(explanation: &'a str) -> Self {
        MutationInstruction {
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

    fn crud_operation(mut self, crud_operation: CRUD) -> Self {
        self.crud_operation = crud_operation;
        self
    }
}

// Flow rate
// queue health


fn mutations<'a>() -> Vec<MutationInstruction<'a>> {
    vec![
        //Think how to do this better
        // MutationInstruction::new("GET resource with no optional parameters")
        // .query_params(ParamMutation::None)
        // .expected(StatusCode::OK)
        // .crud_operations(CRUD::Show， CRUD::Index),
        MutationInstruction::new("GET with no optional parameters")
            .query_params(ParamMutation::None)
            .expected(StatusCode::OK),
        MutationInstruction::new("GET with extra known optional and proper parameters")
            .query_params(ParamMutation::Proper)
            .expected(StatusCode::OK),
        MutationInstruction::new("GET with extra unknown parameters <trusmis=mumi>")
            .query_params(ParamMutation::static_values("trusmis", "mumi"))
            .expected(StatusCode::OK),
        MutationInstruction::new("GET with extra known optional but with improper parameters")
            .query_params(ParamMutation::Wrong)
            .expected(StatusCode::UNPROCESSABLE_ENTITY),
// Applicable to all endpoints
        MutationInstruction::new("GET with wrong content-type <jason>")
            .content_type("minosTest/jason")
            .expected(StatusCode::NOT_ACCEPTABLE),
        MutationInstruction::new("Request with wrong method <TRACE>")
            .method("TRACE")
            .expected(StatusCode::METHOD_NOT_ALLOWED),
    ]
}
