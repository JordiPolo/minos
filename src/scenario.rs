use crate::mutation_instructions;
use crate::operation;
use crate::service::Request;
use openapi_utils::OperationExt;
use reqwest::StatusCode;

#[derive(Debug)]
pub struct ScenarioExecution {
    pub scenario: Scenario,
    pub request: Option<Request>,
}

impl ScenarioExecution {
    pub fn expected_status_code(&self) -> StatusCode {
        self.scenario.instructions.expected
    }
    pub fn expected_body(&self) -> Option<&openapiv3::Response> {
        self.scenario
            .endpoint
            .method
            .response(self.scenario.instructions.expected.as_u16())
    }
}

#[derive(Debug)]
pub struct Scenario {
    pub endpoint: operation::Endpoint,
    pub instructions: mutation_instructions::MutationInstruction,
}

impl Scenario {
    pub fn new(
        endpoint: operation::Endpoint,
        instructions: mutation_instructions::MutationInstruction,
    ) -> Self {
        Scenario {
            endpoint,
            instructions,
        }
    }
}
