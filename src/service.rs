use reqwest;

use std::{
    fmt,
    process::{Child, Command},
    thread::sleep,
    time::Duration,
};

use crate::cli_args::*;
use reqwest::header::ACCEPT;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Method;

// TODO: tuple struct?
// This is the query param as used by the service and in requests to make real calls.
pub type QueryParam = (String, String);

#[derive(Debug)]
pub struct Request<'a> {
    path: String,
    content_type: &'a str,
    method: &'a str,
    query_params: Vec<QueryParam>,
}

impl<'a> Request<'a> {
    pub fn new() -> Request<'a> {
        Request {
            path: "".to_string(),
            content_type: "json",
            method: "get",
            query_params: vec![],
        }
    }

    pub fn path(mut self, new_path: String) -> Self {
        self.path = new_path;
        self
    }

    pub fn query_params(mut self, params: Vec<QueryParam>) -> Self {
        self.query_params = params;
        self
    }

    pub fn content_type(mut self, content_type: &'a str) -> Self {
        self.content_type = content_type;
        self
    }

    pub fn set_method(mut self, method: &'a str) -> Self {
        self.method = method;
        self
    }

    pub fn method(&self) -> Method {
        Method::from_bytes(self.method.as_bytes()).unwrap()
    }

    fn is_method_with_data(&self) -> bool {
        self.method() == Method::PATCH
            || self.method() == Method::POST
            || self.method() == Method::PUT
    }

    pub fn headers(&self) -> HeaderMap {
        let mut request_headers = HeaderMap::new();
        request_headers.insert("x-mws-authentication", HeaderValue::from_static("MWS 5ff4257e-9c16-11e0-b048-0026bbfffe5e:ThJT3EMI7yjhWQVoTEHYQb15/BIKcgTsCnXlBFKTEnezl9bJQhnNRynE+dskJfiWSanCQOAB9/1e+IHk1U1FjKGPe4y"));
        if self.is_method_with_data() {
            request_headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_str(self.content_type).unwrap(),
            );
        }
        request_headers.insert(ACCEPT, HeaderValue::from_str(self.content_type).unwrap());
        request_headers
    }

    pub fn path_and_query(&self) -> String {
        let mut param_string = String::new();
        if !self.query_params.is_empty() {
            param_string = "?".to_string();
            for query_param in self.query_params.iter() {
                param_string.push_str(&format!("{}={}&", query_param.0, query_param.1));
            }
            let len = param_string.len();
            param_string.truncate(len - 1);
        }
        format!("{}{}", self.path, param_string)
    }

    pub fn url(&self, base_url: &str, base_path: &str) -> String {
        format!("{}{}{}", base_url, base_path, self.path_and_query())
    }
}

impl<'a> fmt::Display for Request<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.method, self.path_and_query())
    }
}

pub struct Service {
    base_url: String,
    base_path: String,
    server: Option<Child>,
    client: reqwest::blocking::Client,
}

pub struct ServiceResponse {
    pub status: reqwest::StatusCode,
    pub body: serde_json::Result<serde_json::Value>,
}

impl Drop for Service {
    fn drop(&mut self) {
        self.server.as_mut().map(|s| {
            s.kill()
                .expect("Service could not be killed, maybe it was not running (crashed?)")
        });
    }
}

impl Service {
    pub fn new(config: &CLIArgs, base_path: String) -> Self {
        let server_command: Vec<&str> = config.server_command.split(' ').collect();
        let (command, arguments) = server_command.split_at(1);

        let server = if config.server_run {
            let server = Command::new(command[0])
                .args(arguments)
                .spawn()
                .expect("failed to execute the server.");
            println!("Starting server. Waiting {:?} seconds", &config.server_wait);
            sleep(Duration::from_millis(config.server_wait * 1000));
            Some(server)
        } else {
            None
        };
    //    let server = None;

        let client = reqwest::blocking::Client::new();
        Service {
            base_url: config.base_url.clone(),
            base_path,
            server,
            client,
        }
    }

    pub fn send(&self, request: &Request) -> ServiceResponse {
        let endpoint = request.url(&self.base_url, &self.base_path);
        println!("{:?}", endpoint);
        // TODO: debugging mode
        //   println!("{:?}", request.headers());
        let resp = self
            .client
            .request(request.method(), &endpoint)
            .headers(request.headers())
            .send()
            .expect("The request to the endpoint failed.");

        let status = resp.status();

        let body = resp
            .text()
            .expect("It was not possible to read data from body.");
        // TODO: This should be ok on debug mode.
        // for header in resp.headers() {
        //     println!("response header {:?}", header);
        // }
        ServiceResponse {
            status,
            body: serde_json::from_str(&body),
        }
    }
}
