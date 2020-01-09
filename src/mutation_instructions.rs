use crate::operation::CRUD;
use crate::request_param::RequestParam;
use reqwest::StatusCode;

#[derive(Debug)]
pub struct MutationInstruction {
    pub content_type: Option<String>,
    pub method: Option<String>,
    pub crud_operation: Option<CRUD>,
    pub path_params: PathMutation,  // TODO allow to mutate the path
    pub required_params: ParamMutation, // Required parameters in query string
    pub query_params: ParamMutation,    // Optional query parameters
    pub expected: StatusCode,
    pub explanation: String,
}

#[derive(Debug, PartialEq)]
pub enum PathMutation {
    Proper,
    Random,
}

#[derive(Debug, PartialEq)]
pub enum ParamMutation {
    Static(RequestParam),
    Proper,
    Wrong,
    None,
    //    Empty, what would happen in this case?
}

pub fn mutations() -> Vec<MutationInstruction> {
    vec![
        // TODO: Read all this from a file
        MutationInstruction::new(
            "Request with proper required parameters and no optional parameters",
        )
        .expected(StatusCode::OK),
        MutationInstruction::new("Request without the needed required parameters")
            .required_params(ParamMutation::None)
            .expected(StatusCode::UNPROCESSABLE_ENTITY),
        MutationInstruction::new("Request with incorrect required parameters")
            .required_params(ParamMutation::Wrong)
            .expected(StatusCode::UNPROCESSABLE_ENTITY),

        MutationInstruction::new("Request with unknown id")
            .path_params(PathMutation::Random)
            .expected(StatusCode::NOT_FOUND),

        MutationInstruction::new("Request with extra known optional and proper parameters")
            .query_params(ParamMutation::Proper)
            .expected(StatusCode::OK),
        MutationInstruction::new("Request with extra known optional but with improper parameters")
            .query_params(ParamMutation::Wrong)
            .expected(StatusCode::UNPROCESSABLE_ENTITY),

        MutationInstruction::new("Request with extra unknown parameters <trusmis=mumi>")
            .query_params(ParamMutation::static_values("trusmis", "mumi"))
            .expected(StatusCode::OK),
        MutationInstruction::new("Request with wrong content-type <jason>")
            .content_type("minosTest/jason")
            .expected(StatusCode::NOT_ACCEPTABLE),
        MutationInstruction::new("Request with wrong method <TRACE>")
            .method("TRACE")
            .expected(StatusCode::NOT_FOUND),
        //  .expected(StatusCode::METHOD_NOT_ALLOWED), Technically this status is more correct TODO
    ]
}

impl ParamMutation {
    fn static_values(name: &str, value: &str) -> Self {
        ParamMutation::Static(RequestParam::new(name, value))
    }
}

impl MutationInstruction {
    fn new(explanation: &str) -> Self {
        MutationInstruction {
            content_type: None,
            method: None,
            crud_operation: None,
            required_params: ParamMutation::Proper,
            query_params: ParamMutation::None,
            path_params: PathMutation::Proper,
            expected: StatusCode::OK,
            explanation: explanation.to_string(),
        }
    }
    fn content_type(mut self, content_type: &str) -> Self {
        self.content_type = Some(content_type.to_string());
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

    fn required_params(mut self, required_params: ParamMutation) -> Self {
        self.required_params = required_params;
        self
    }

    fn path_params(mut self, path_params: PathMutation) -> Self {
        self.path_params = path_params;
        self
    }

    fn method(mut self, method: &str) -> Self {
        self.method = Some(method.to_string());
        self
    }
}
