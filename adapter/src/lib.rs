#![feature(ascii_char)]

//! Axum Cloudflare Adapter
//!
//! A collection of tools allowing Axum to be run within a Cloudflare worker. See example usage below.
//!

//! ```
//! use worker::*;
//!
//! use axum::{
//! 		response::{Html},
//! 		routing::get,
//! 		Router as AxumRouter,
//!         extract::State,
//! };
//! use axum_cloudflare_adapter::{to_axum_request, to_worker_response, wasm_compat, EnvWrapper};
//! use tower_service::Service;
//! use std::ops::Deref;
//!
//! #[derive(Clone)]
//! pub struct AxumState {
//!    	pub env_wrapper: EnvWrapper,
//! }
//!
//! #[wasm_compat]
//! async fn index(State(state): State<AxumState>) -> Html<&'static str> {
//! 		let env: &Env = state.env_wrapper.env.deref();
//! 		let worker_rs_version: Var = env.var("WORKERS_RS_VERSION").unwrap();
//!         console_log!("WORKERS_RS_VERSION: {}", worker_rs_version.to_string());
//!
//! 		Html("<p>Hello from Axum!</p>")
//! }
//!
//! #[event(fetch)]
//! async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
//!         let mut router: AxumRouter = AxumRouter::new()
//! 				.route("/", get(index))
//!                 .with_state(AxumState {
//! 				    env_wrapper: EnvWrapper::new(env),
//! 		        });
//!
//! 		let axum_request = to_axum_request(req).await.unwrap();
//! 		let axum_response = router.call(axum_request).await.unwrap();
//! 		let response = to_worker_response(axum_response).await.unwrap();
//!
//! 		Ok(response)
//! }
//!
//! ```
mod error;

use std::str::FromStr;
use std::sync::Arc;
use axum::{
    body::Body,
    http::{Method, Request, Uri},
    http::header::HeaderName,
    response::Response,
};
use futures::TryStreamExt;
use worker::{
    Request as WorkerRequest,
    Response as WorkerResponse,
    Headers,
};
pub use error::Error;

pub async fn to_axum_request(mut worker_request: WorkerRequest) -> Result<Request<Body>, Error> {
    let method = Method::from_bytes(worker_request.method().to_string().as_bytes())?;

    let uri = Uri::from_str(worker_request.url()?
        .to_string()
        .as_str())?;

    let body = worker_request.bytes().await?;


    let mut http_request = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::from(body))?;


    for (header_name, header_value) in worker_request.headers() {
        http_request.headers_mut().insert(
            HeaderName::from_str(header_name.as_str())?,
            header_value.parse()?,
        );
    }

    Ok(http_request)
}

pub async fn to_worker_response(response: Response<Body>) -> Result<WorkerResponse, Error> {
    let mut bytes: Vec<u8> = Vec::<u8>::new();

    let (parts, body) = response.into_parts();

    let mut stream = body.into_data_stream();
    while let Some(chunk) = stream.try_next().await? {
        bytes.extend_from_slice(&chunk);
    }

    let code = parts.status.as_u16();

    let mut worker_response = WorkerResponse::from_bytes(bytes)?;
    worker_response = worker_response.with_status(code);

    let mut headers = Headers::new();
    for (key, value) in parts.headers.iter() {
        headers.set(
            key.as_str(),
            value.to_str()?,
        ).unwrap()
    }
    worker_response = worker_response.with_headers(headers);

    Ok(worker_response)
}

pub use axum_wasm_macros::wasm_compat;

#[derive(Clone)]
pub struct EnvWrapper {
    pub env: Arc<worker::Env>,
}

impl EnvWrapper {
    pub fn new(env: worker::Env) -> Self {
        Self {
            env: Arc::new(env),
        }
    }
}

unsafe impl Send for EnvWrapper {}

unsafe impl Sync for EnvWrapper {}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        response::Html,
        response::IntoResponse,
    };
    use wasm_bindgen_test::{*};
    use worker::{RequestInit, ResponseBody, Method as WorkerMethod};
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn it_should_convert_the_worker_request_to_an_axum_request() {
        let mut request_init = RequestInit::new();
        let mut headers = Headers::new();
        headers.append("Content-Type", "text/html").unwrap();
        headers.append("Cache-Control", "no-cache").unwrap();
        request_init.with_headers(headers);
        request_init.with_method(WorkerMethod::Get);
        let worker_request = WorkerRequest::new_with_init("https://logankeenan.com", &request_init).unwrap();

        let request = to_axum_request(worker_request).await.unwrap();

        assert_eq!(request.uri(), "https://logankeenan.com");
        assert_eq!(request.method(), "GET");
        assert_eq!(request.headers().get("Content-Type").unwrap(), "text/html");
        assert_eq!(request.headers().get("Cache-Control").unwrap(), "no-cache");
    }

    #[wasm_bindgen_test]
    async fn it_should_convert_the_worker_request_to_an_axum_request_with_a_body() {
        let mut request_init = RequestInit::new();
        request_init.with_body(Some("hello world!".into()));
        request_init.with_method(WorkerMethod::Post);
        let worker_request = WorkerRequest::new_with_init("https://logankeenan.com", &request_init).unwrap();

        let request = to_axum_request(worker_request).await.unwrap();


        let mut bytes: Vec<u8> = Vec::<u8>::new();

        let mut stream = request.into_body().into_data_stream();
        while let Some(chunk) = stream.try_next().await.unwrap() {
            bytes.extend_from_slice(&chunk);
        }

        assert_eq!(bytes.to_vec(), b"hello world!");
    }

    #[wasm_bindgen_test]
    async fn it_should_convert_the_axum_response_to_a_worker_response() {
        let response = Html::from("Hello World!").into_response();
        let worker_response = to_worker_response(response).await.unwrap();

        assert_eq!(worker_response.status_code(), 200);
        assert_eq!(worker_response.headers().get("Content-Type").unwrap().unwrap(), "text/html; charset=utf-8");
        let body = match worker_response.body() {
            ResponseBody::Body(body) => body.clone(),
            _ => vec![]
        };
        assert_eq!(body, b"Hello World!");
    }

    #[wasm_bindgen_test]
    async fn it_should_convert_the_axum_response_to_a_worker_response_with_an_empty_body() {
        let body = Body::empty();
        let response = Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(body)
            .unwrap();


        let worker_response = to_worker_response(response).await.unwrap();

        assert_eq!(worker_response.status_code(), 200);
        assert_eq!(worker_response.headers().get("Content-Type").unwrap().unwrap(), "text/html");
        let body = match worker_response.body() {
            ResponseBody::Body(body) => body.clone(),
            _ => b"should be empty".to_vec()
        };
        assert_eq!(body.len(), 0);
    }
}
