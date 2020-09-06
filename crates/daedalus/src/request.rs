use crate::request_param::RequestParam;
use http::header::{HeaderValue, ACCEPT, CONTENT_TYPE};
use http::Method;

// This is the only thing users of this library will see
pub type Request = http::Request<&'static str>;

// This is the struct that we will store on each scenario.
// Note that http::Request does not have a clone method
// But the user of this library may take the request, modify it
// And run it multiple times.
// To allow them to do whatever they want, we provide a copy
#[derive(Debug)]
pub(crate) struct ScenarioRequest {
    request: http::Request<&'static str>,
}

impl ScenarioRequest {
    pub(crate) fn clone(&self) -> Request {
        let mut clone = http::Request::new("");
        *clone.method_mut() = self.request.method().clone();
        *clone.uri_mut() = self.request.uri().clone();
        *clone.version_mut() = self.request.version();
        *clone.headers_mut() = self.request.headers().clone();
        // Self::headers(&mut clone.headers_mut());
        clone
    }
}

// This is a builder we use to create a request from a given mutation.
#[derive(Debug)]
pub(crate) struct RequestBuilder {
    path: String,
    content_type: String,
    method: String,
    query_params: Vec<RequestParam>,
}

impl RequestBuilder {
    pub fn new() -> RequestBuilder {
        RequestBuilder {
            path: "".to_string(),
            content_type: "json".to_string(),
            method: "get".to_string(),
            query_params: vec![],
        }
    }

    pub fn path(&mut self, new_path: String) -> &mut Self {
        self.path = new_path;
        self
    }

    pub fn query_params(&mut self, params: RequestParam) -> &mut Self {
        self.query_params.push(params);
        self
    }

    pub fn content_type(&mut self, content_type: String) -> &mut Self {
        self.content_type = content_type;
        self
    }

    pub fn method(&mut self, method: String) -> &mut Self {
        self.method = method;
        self
    }

    pub fn build(self) -> ScenarioRequest {
        let mut builder = http::request::Builder::new();

        let headers = builder.headers_mut().unwrap();
        headers.insert(ACCEPT, HeaderValue::from_str(&self.content_type).unwrap());
        if self.is_method_with_data() {
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_str(&self.content_type).unwrap(),
            );
        }

        let request = builder
            .method(self.to_http_method())
            .uri(&self.path_and_query())
            .body("")
            .unwrap();

        ScenarioRequest { request }
    }

    fn path_and_query(&self) -> String {
        let mut param_string = String::new();
        if !self.query_params.is_empty() {
            param_string = "?".to_string();
            for query_param in self.query_params.iter() {
                if query_param.value.is_some() {
                    param_string.push_str(&format!(
                        "{}={}&",
                        query_param.name,
                        query_param.value.as_ref().unwrap()
                    ));
                }
            }
            let len = param_string.len();
            param_string.truncate(len - 1);
        }
        format!("{}{}", self.path, param_string)
    }

    fn to_http_method(&self) -> Method {
        Method::from_bytes(self.method.as_bytes()).unwrap()
    }

    fn is_method_with_data(&self) -> bool {
        let my_method = self.to_http_method();
        my_method == Method::PATCH || my_method == Method::POST || my_method == Method::PUT
    }
}
