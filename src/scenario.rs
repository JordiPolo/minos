use crate::mutation::Mutation;
use crate::operation;
use crate::service::Request;
use openapi_utils::OperationExt;
use reqwest::StatusCode;

#[derive(Debug)]
pub struct Scenario {
    pub endpoint: operation::Endpoint,
    pub instructions: Vec<Mutation>,
    pub request: Request,
}

#[derive(Debug)]
pub struct ScenarioExpectation<'a> {
    pub status_code: StatusCode,
    pub body: Option<&'a openapiv3::Response>,
    pub content_type: String,
}

impl Scenario {
    pub fn new(
        endpoint: operation::Endpoint,
        instructions: Vec<Mutation>,
        request: Request,
    ) -> Self {
        //, subject: ScenarioSubject) -> Self {
        Scenario {
            endpoint,
            instructions,
            request,
            //   subject,
        }
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


// impl From<Vec<Mutation>> for Request {
//     fn from(mutations: Vec<Mutation>) -> Self {
//         let mut request = Request::new();
//         let mut query_params = Vec::new();
//         for mutation in mutations {
//             match mutation.mutagen.request_part {
//                 RequestPart::ContentType => {
//                     request = request.content_type(mutation.value.clone().unwrap())
//                 }
//                 RequestPart::Method => {
//                     request = request.set_method(mutation.value.clone().unwrap())
//                 }
//                 RequestPart::Path => request = request.path(mutation.value.clone().unwrap()),
//                 RequestPart::AnyParam => query_params.push(mutation.param_value.clone().unwrap()),
//                 _ => {} //unimplemented!("We do not know how to mutate this endpoint level item. {:?}", instruction.request_part),
//             }
//         }
//         request = request.query_params(query_params);
//         request
//     }
// }
