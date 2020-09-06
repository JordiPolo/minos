use crate::mutation::Mutation;
use crate::operation;
use crate::request::{Request, ScenarioRequest};
use http::StatusCode;
use openapi_utils::OperationExt;

#[derive(Debug)]
pub struct Scenario<'a> {
    pub endpoint: &'a operation::Endpoint,
    pub instructions: Vec<Mutation>,
    request: ScenarioRequest,
}

#[derive(Debug)]
pub struct ScenarioExpectation<'a> {
    pub status_code: StatusCode,
    pub body: Option<&'a openapiv3::Response>,
    pub content_type: String,
}

impl<'a> Scenario<'a> {
    pub(crate) fn new(
        endpoint: &'a operation::Endpoint,
        instructions: Vec<Mutation>,
        request: ScenarioRequest,
    ) -> Self {
        //, subject: ScenarioSubject) -> Self {
        Scenario {
            endpoint,
            instructions,
            request,
            //   subject,
        }
    }

    pub fn request(&self) -> Request {
        //http::Request<&'static str> {
        self.request.clone()
    }

    pub fn expectation(&self) -> ScenarioExpectation<'_> {
        ScenarioExpectation {
            status_code: self.expected_status_code(),
            body: self
                .endpoint
                .method
                .response(self.expected_status_code().as_u16()),
            content_type: String::from("application/json"),
        }
    }

    fn expected_status_code(&self) -> StatusCode {
        for instruction in self.instructions.clone() {
            if instruction.mutagen.expected != StatusCode::OK {
                return instruction.mutagen.expected;
            }
        }
        StatusCode::OK
    }
}
