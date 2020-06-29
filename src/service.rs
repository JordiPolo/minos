use std::{
    fmt,
    process::{Child, Command},
    thread::sleep,
    time::Duration,
};

use crate::cli_args::*;
use crate::request_param::RequestParam;
use log::{debug, info};
use rand::Rng;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::Method;
//use mauth_client::*;

#[derive(Debug)]
pub struct Request {
    path: String,
    content_type: String,
    method: String,
    query_params: Vec<RequestParam>,
}

impl Request {
    pub fn new() -> Request {
        Request {
            path: "".to_string(),
            content_type: "json".to_string(),
            method: "get".to_string(),
            query_params: vec![],
        }
    }

    pub fn path(mut self, new_path: String) -> Self {
        self.path = new_path;
        self
    }

    pub fn query_params(mut self, params: Vec<RequestParam>) -> Self {
        self.query_params = params;
        self
    }

    pub fn content_type(mut self, content_type: String) -> Self {
        self.content_type = content_type;
        self
    }

    pub fn set_method(mut self, method: String) -> Self {
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
        let mut rng = rand::thread_rng();
        let trace_id = format!("{:x}", rng.gen::<u128>());
        let span_id = format!("{:x}", rng.gen::<u64>());
// TODO: Verify apps receive and use thsese keys
        request_headers.insert("Accept", HeaderValue::from_static("application/json"));
        request_headers.insert("X-B3-Sampled", HeaderValue::from_static("0"));
        request_headers.insert("X-B3-TraceId", HeaderValue::from_str(&trace_id).unwrap());
        request_headers.insert("X-B3-SpanId", HeaderValue::from_str(&span_id).unwrap());

        if self.is_method_with_data() {
            request_headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_str(&self.content_type).unwrap(),
            );
        }
        request_headers.insert(ACCEPT, HeaderValue::from_str(&self.content_type).unwrap());
        request_headers
    }

    pub fn path_and_query(&self) -> String {
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

    pub fn url(&self, base_url: &str, base_path: &str) -> String {
        format!("{}{}{}", base_url, base_path, self.path_and_query())
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.method, self.path_and_query())
    }
}

pub struct Service {
    base_url: String,
    base_path: String,
    server: Option<Child>,
    client: reqwest::Client, //reqwest::blocking::Client,
}

pub struct ServiceResponse {
    pub status: reqwest::StatusCode,
    pub body: serde_json::Result<serde_json::Value>,
    pub content_type: Option<String>,
}

impl Drop for Service {
    fn drop(&mut self) {
        if let Some(server) = self.server.as_mut() {
            server
                .kill()
                .expect("Service could not be killed, maybe it was not running (crashed?)");
        }
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

        let client = reqwest::Client::new();
        Service {
            base_url: config.base_url.clone(),
            base_path,
            server,
            client,
        }
    }

    pub fn build_hyper_request(&self, request: &Request) -> http::request::Request<hyper::Body> {
        let endpoint = request.url(&self.base_url, &self.base_path);
        info!("Sending request {:?}", request);
        debug!("Request headers {:?}", request.headers());


        // Create http request
        let mut builder = http::request::Builder::new();

        let headers = builder.headers_mut().unwrap();
         for (key, value) in request.headers().iter() {
             headers.insert(key, value.clone());
         }

        let mut requ = builder.method(request.method())
        .uri(&endpoint)
        .body(hyper::Body::from(""))
        .expect(&format!("{:?} is not a valid URL. Check the base URL.", &endpoint));


        // TODO: Do not re-read the file multiple times
        // Add mauth headers
        // let mauth_info = MAuthInfo::from_default_file().expect("Mauth file missing");
        // // on empy body we digest "" TODO: Support request bodies
        // let (_, body_digest) = MAuthInfo::build_body_with_digest("".to_string());
        // mauth_info.sign_request_v2(&mut requ, &body_digest);
        requ
    }

// TODO: when network does not find the address this is blocking the thread
    pub async fn send(&self, request: &Request) -> Result<ServiceResponse, reqwest::Error> {
        let requ = self.build_hyper_request(request);

         //launch request as a request request wich implies copying, TODO: prevent copying
        let resp = self
        .client
        .request(requ.method().clone(), reqwest::Url::parse(&requ.uri().to_string()).unwrap())
        .headers(requ.headers().clone())
        .send().await?;

        debug!("Response headers {:?}", resp.headers());

        let status = resp.status();
        let content_type = resp.headers().get(http::header::CONTENT_TYPE).map(|h| {
            String::from(
                h.to_str()
                    .expect("Non ASCII characters found in your content-type header."),
            )
        });
        let body = resp.text().await?;

        Ok(ServiceResponse {
            status,
            body: serde_json::from_str(&body),
            content_type,
        })
    }
}
