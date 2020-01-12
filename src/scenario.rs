use crate::mutation_instructions::MutationInstruction;
use crate::operation;
use crate::service::Request;
use openapi_utils::OperationExt;
use reqwest::StatusCode;

#[derive(Debug)]
pub struct ScenarioExecution {
    pub scenario: Scenario,
    pub request: Option<Request>,
    pub passed: bool,
}

impl ScenarioExecution {
    pub fn new(scenario: Scenario, request: Option<Request>) -> Self {
        ScenarioExecution {
            scenario,
            request,
            passed: false,
        }
    }
}

#[derive(Debug)]
pub struct ScenarioExpectation<'a> {
    pub status_code: StatusCode,
    pub body: Option<&'a openapiv3::Response>,
    pub content_type: String,
}

#[derive(Debug)]
pub struct Scenario {
    pub endpoint: operation::Endpoint,
    pub instructions: MutationInstruction,
}

impl Scenario {
    pub fn new(endpoint: operation::Endpoint, instructions: MutationInstruction) -> Self {
        Scenario {
            endpoint,
            instructions,
        }
    }

    pub fn expectation<'a>(&'a self) -> ScenarioExpectation<'a> {
        ScenarioExpectation {
            status_code: self.instructions.expected,
            body: self
                .endpoint
                .method
                .response(self.instructions.expected.as_u16()),
            content_type: String::from("application/json"),
        }
    }
}
