use json;
use reqwest;

use std::{
    process::{Child, Command},
    thread::sleep,
    time::Duration,
};

use cli_args::*;
use reqwest::header::CONTENT_TYPE;
use reqwest::Method;

// TODO: tuple struct?
// This is the query param as used by the service and in requests to make real calls.
pub type QueryParam = (String, String);

pub struct Service {
    base_url: String,
    base_path: String,
    server: Option<Child>,
    client: reqwest::Client,
}

pub struct ServiceResponse {
    //pub status: String,
    pub status: reqwest::StatusCode,
    pub value: Result<json::JsonValue, json::Error>,
}

pub struct Request<'a> {
    pub path: &'a str,
    pub content_type: &'a str,
    pub method: &'a str,
    pub query_params: Vec<QueryParam>,
    pub path_params: Vec<String>,
}

impl<'a> Request<'a> {
    pub fn new() -> Request<'a> {
        Request {
            path: "",
            content_type: "json",
            method: "get",
            query_params: vec![],
            path_params: vec![],
        }
    }

    pub fn path(mut self, new_path: &'a str) -> Self {
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

    pub fn path_params(mut self, path_params: Vec<String>) -> Self {
        self.path_params = path_params;
        self
    }

    pub fn method(&self) -> Method {
        Method::from_bytes(self.method.as_bytes()).unwrap()
    }

    pub fn url(&self, base_url: &str, base_path: &str) -> String {
        let endpoint = self.endpoint(base_url, base_path);
        let mut param_string = String::new();
        for query_param in self.query_params.iter() {
            param_string = format!("{}={}", query_param.0, query_param.1);
        }
        format!("{}?{}", endpoint, param_string)
    }

    //TODO: Content-type should be a header, not an extension
    pub fn endpoint(&self, base_url: &str, base_path: &str) -> String {
        format!(
            "{}{}{}.{}",
            base_url, base_path, self.path, self.content_type
        )
    }
}

impl Service {
    pub fn new(config: &CLIArgs, base_path_option: &Option<String>) -> Self {
        //, server: Vec<&str>, server_wait: u64) -> Self {
        let server_command: Vec<&str> = config.server_command.split(" ").collect();
        let (command, arguments) = server_command.split_at(1);

        //TODO: make this work again
        let server = None;
        // let server = if config.server_run {
        //     Some(Command::new(command[0]).args(arguments).spawn().expect("failed to execute the server."))
        // } else {
        //     None
        // };

        //   println!("Waiting {:?} seconds", &config.server_wait);
        //   server.as_ref().map(|_| sleep(Duration::from_millis(config.server_wait * 1000)));

        let base_path = base_path_option.clone().unwrap_or("".to_string());
        let client = reqwest::Client::new();
        Service {
            base_url: config.base_url.clone(),
            base_path,
            server,
            client,
        }
    }

    // pub fn kill(&mut self) {
    //     self.server.as_mut().map(|s| {
    //         s.kill()
    //             .expect("Service could not be killed, maybe it was not running (crashed?)")
    //     });
    // }

    pub fn send(&self, request: &Request) -> ServiceResponse {
        let endpoint = request.url(&self.base_url, &self.base_path);
        let mut resp = self
            .client
            .request(request.method(), &endpoint)
            .header(CONTENT_TYPE, request.content_type)
            .header("x-mws-authentication", "MWS 5ff4257e-9c16-11e0-b048-0026bbfffe5e:ThJT3EMI7yjhWQVoTEHYQb15/BIKcgTsCnXlBFKTEnezl9bJQhnNRynE+dskJfiWSanCQOAB9/1e+IHk1U1FjKGPe4y")
            .send()
            .expect("The request to the endpoint failed.");
        let body = resp
            .text()
            .expect("It was not possible to read data from body.");
        ServiceResponse {
            status: resp.status(),
            value: json::parse(&body),
        }
    }

    // pub fn call_success(&self, path: &str, query_params: Option<QueryParam>) -> ServiceResponse {
    //     let endpoint = self.endpoint_param(path, "json", query_params);

    //     let mut resp = self.client.get(&endpoint).send().expect("The request to the endpoint failed.");
    //     let body = resp.text().expect(
    //         "It was not possible to read data from body.",
    //     );

    //     ServiceResponse { status: resp.status(), value: json::parse(&body) }
    // }

    // pub fn call_failed_content_type(&self, path: &str) -> ServiceResponse {
    //     let endpoint = self.endpoint(path, "jason");
    // //    println!("Calling {:?}\n", endpoint);

    //     let mut resp = self.client.get(&endpoint).send().expect("The request to the endpoint failed.");
    //     let body = resp.text().expect(
    //         "It was not possible to read data from body.",
    //     );
    // //    json::parse(&body).unwrap()
    //     ServiceResponse { status: resp.status(), value: json::parse(&body) }
    // }

    // pub fn call_with_method(&self, path: &str, method_name: &str) -> ServiceResponse {
    //     let endpoint = self.endpoint(path, "json");
    // //    println!("Calling {:?}\n", endpoint);
    //     let resp = if method_name == "patch" {
    //       self.client.patch(&endpoint).send().expect("The request to the endpoint failed.")
    //     } else {
    //       self.client.put(&endpoint).send().expect("The request to the endpoint failed.")
    //     };

    //     ServiceResponse { status: resp.status(), value: Ok(json::JsonValue::Null) }
    // }
}
