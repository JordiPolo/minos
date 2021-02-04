//use std::fmt;
//use crate::cli_args::*;
use crate::authentication::Authentication;
use rand::Rng;
use reqwest::header::{HeaderMap, HeaderValue};
use tracing::debug;
//use reqwest::Method;
use daedalus::Request;

#[derive(Debug)]
pub struct RunnableRequest {
    request: http::request::Request<hyper::Body>,
}

impl RunnableRequest {
    fn new(minos_request: http::request::Request<hyper::Body>) -> Self {
        let mut request = minos_request; //.http_request();
        Self::headers(&mut request.headers_mut());
        RunnableRequest { request }
    }

    pub fn http_request(self) -> http::request::Request<hyper::Body> {
        self.request
    }

    pub fn trace_id(&self) -> &str {
        self.request.headers()["X-B3-TraceID"].to_str().unwrap()
    }

    fn headers(request_headers: &mut HeaderMap<HeaderValue>) {
        let mut rng = rand::thread_rng();
        let trace_id = format!("{:x}", rng.gen::<u128>());
        let span_id = format!("{:x}", rng.gen::<u64>());
        // TODO: Verify apps receive and use thsese keys
        request_headers.insert("X-B3-Sampled", HeaderValue::from_static("0"));
        request_headers.insert("X-B3-TraceId", HeaderValue::from_str(&trace_id).unwrap());
        request_headers.insert("X-B3-SpanId", HeaderValue::from_str(&span_id).unwrap());
    }
}

pub struct Service {
    base_url: String,
    client: reqwest::Client, //reqwest::blocking::Client,
    authentication: Authentication,
}

pub struct ServiceResponse {
    pub status: reqwest::StatusCode,
    pub body: serde_json::Result<serde_json::Value>,
    pub content_type: Option<String>,
}

impl Service {
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::new();
        let authentication = Authentication::new();
        Service {
            base_url: base_url.to_owned(),
            client,
            authentication,
        }
    }
    pub fn runnable_request(&self, minos_request: Request) -> RunnableRequest {
        let mut request = minos_request;
        *request.uri_mut() = format!("{}{}", self.base_url, request.uri())
            .parse()
            .unwrap();
        let mut request = request.map(hyper::Body::from);
        self.authentication.authenticate(&mut request);
        RunnableRequest::new(request)
    }

    // TODO: when network does not find the address this is blocking the thread
    pub async fn send(&self, request: RunnableRequest) -> Result<ServiceResponse, reqwest::Error> {
        // TODO: ugly
        let requ = request.http_request();
        debug!("Sending request {:?}", requ);
        debug!("Request headers {:?}", requ.headers());

        let resp = self
            .client
            .request(
                requ.method().clone(),
                reqwest::Url::parse(&requ.uri().to_string()).unwrap(),
            )
            .headers(requ.headers().clone())
            .send()
            .await?;

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
