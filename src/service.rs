use reqwest;
use json;

use std::{thread::sleep, time::Duration, process::{Command, Child}};

use cli_args::*;


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

impl Service {
    pub fn new(config: &CLIArgs, base_path_option: &Option<String>) -> Self { //, server: Vec<&str>, server_wait: u64) -> Self {
        let server_command : Vec<&str> = config.server_command.split(" ").collect();
        let (command, arguments) = server_command.split_at(1);

        let server = if config.server_run {
            Some(Command::new(command[0]).args(arguments).spawn().expect("failed to execute the server."))
        } else {
            None
        };
        server.as_ref().map(|_| sleep(Duration::from_millis(config.server_wait * 1000)));

        let base_path = base_path_option.clone().unwrap_or("".to_string());
        let client = reqwest::Client::new();
        Service {base_url: config.base_url.clone(), base_path, server, client}
    }

    pub fn kill(&mut self) {
        self.server.as_mut().map(|s| s.kill().expect("Service could not be killed, maybe it was not running (crashed?)"));
    }

    pub fn call_success(&self, path: &str, query_params: Option<(&str, &str)>) -> ServiceResponse {
        let endpoint = self.endpoint_param(path, "json", query_params);
    //    println!("Calling {:?}\n", endpoint);

        let mut resp = self.client.get(&endpoint).send().expect("The request to the endpoint failed.");
        let body = resp.text().expect(
            "It was not possible to read data from body.",
        );
       // assert!(resp.status().is_success(), "Failed response when it should have been a 200.");
        ServiceResponse { status: resp.status(), value: json::parse(&body) }
    }

    pub fn call_failed_content_type(&self, path: &str) -> ServiceResponse {
        let endpoint = self.endpoint(path, "jason");
    //    println!("Calling {:?}\n", endpoint);

        let mut resp = self.client.get(&endpoint).send().expect("The request to the endpoint failed.");
        let body = resp.text().expect(
            "It was not possible to read data from body.",
        );
    //    json::parse(&body).unwrap()
        ServiceResponse { status: resp.status(), value: json::parse(&body) }
    }


    pub fn call_with_method(&self, path: &str, method_name: &str) -> ServiceResponse {
        let endpoint = self.endpoint(path, "json");
    //    println!("Calling {:?}\n", endpoint);
        let resp = if method_name == "patch" {
          self.client.patch(&endpoint).send().expect("The request to the endpoint failed.")
        } else {
          self.client.put(&endpoint).send().expect("The request to the endpoint failed.")
        };

        ServiceResponse { status: resp.status(), value: Ok(json::JsonValue::Null) }
    }

    fn endpoint_param(&self, path: &str, content_type: &str, query_params: Option<(&str, &str)>) -> String {
        let endpoint = self.endpoint(path, content_type);
        match query_params {
            Some((name, value)) => format!("{}?{}={}", endpoint, name, value),
            None => endpoint,
        }
    }

    fn endpoint(&self, path: &str, content_type: &str) -> String {
        format!("{}{}{}.{}", self.base_url, self.base_path, path, content_type)
    }

}