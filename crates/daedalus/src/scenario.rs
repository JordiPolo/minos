use crate::mutation::Mutation;
use crate::operation;
use crate::request::{Request, ScenarioRequest};
use http::StatusCode;
use openapi_utils::OperationExt;
use std::fmt;

/// An auto-generated scenario
#[derive(Debug)]
pub struct Scenario<'a> {
    /// The instructions we used to build this scenario
    pub instructions: Vec<Mutation>,
    request: ScenarioRequest,
    expectation: ScenarioExpectation<'a>,
}

/// The expectation to pass for a given scenario
#[derive(Debug)]
pub struct ScenarioExpectation<'a> {
    /// Expected status code
    pub status_code: StatusCode,
    /// Expected body
    pub body: Option<&'a openapiv3::Response>,
    /// Expected content type
    pub content_type: String,
}

impl fmt::Display for Scenario<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let request = &self.request.request;
        write!(f, "{} {}", request.method(), request.uri())
    }
}

impl<'a> Scenario<'a> {
    pub(crate) fn new(
        endpoint: &'a operation::Endpoint,
        instructions: Vec<Mutation>,
        request: ScenarioRequest,
    ) -> Self {
        let status_code = Self::expected_status_code(&instructions);

        let expectation = ScenarioExpectation {
            status_code,
            body: endpoint.method.response(status_code.as_u16()),
            content_type: String::from("application/json"),
        };

        Scenario {
            instructions,
            request,
            expectation,
        }
    }

    /// Returns a new runnable request from the scenario
    pub fn request(&self) -> Request {
        self.request.clone()
    }

    /// The expectations for this scenario
    pub fn expectation(&self) -> &ScenarioExpectation<'_> {
        &self.expectation
    }

    fn expected_status_code(instructions: &Vec<Mutation>) -> StatusCode {
        match instructions
            .iter()
            .find(|i| i.mutagen.expected != StatusCode::OK)
        {
            Some(instruction) => instruction.mutagen.expected,
            None => StatusCode::OK,
        }
    }
}
