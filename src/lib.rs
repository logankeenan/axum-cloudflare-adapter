//! Axum Cloudflare
//!
//! Simple functions to convert a Cloudflare worker request to an Axum request and
//! then convert the Axum response to a Worker reponse.
//!

//! ```
//! use worker::*;
//!
//! use axum::{
//! 		response::{Html},
//! 		routing::get,
//! 		Router as AxumRouter,
//! };
//! use axum_cloudflare::{to_axum_request, to_worker_response};
//! use tower_service::Service;
//!
//! async fn index() -> Html<&'static str> {
//! 		Html("<p>Hello from Axum!</p>")
//! }
//!
//! #[event(fetch)]
//! pub async fn main(req: Request, _env: Env, _ctx: worker::Context) -> Result<Response> {
//! 		let mut _router: AxumRouter = AxumRouter::new()
//! 				.route("/", get(index));
//!
//! 		let axum_request = to_axum_request(req).await;
//! 		let axum_response = _router.call(axum_request).await.unwrap();
//! 		let response = to_worker_response(axum_response).await;
//!
//! 		Ok(response)
//! }
//!
//! ```

use std::str::FromStr;
use axum::{
    body::Body,
    http::{Method, Request, Uri},
    http::header::HeaderName,
    response::Response,
};
use worker::{
    Request as WorkerRequest,
    Response as WorkerResponse,
    Headers,
};

pub async fn to_axum_request(mut worker_request: WorkerRequest) -> Request<Body> {
    let method = Method::from_str(worker_request.method().to_string().as_str()).expect("Invalid Method");

    let uri = Uri::from_str(worker_request.url().expect("Failed to parse worked URL")
        .to_string()
        .as_str())
        .expect("Failed to create URI");

    let body = worker_request.bytes().await.expect("failed to parse Worker Body");


    let mut http_request = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::from(body))
        .expect("Invalid HTTP request");


    for (header_name, header_value) in worker_request.headers() {
        http_request.headers_mut().insert(
            HeaderName::from_str(header_name.as_str()).expect("Invalid Header Name"),
            header_value.parse().expect("Invalid Header Value"),
        );
    }

    http_request
}

pub async fn to_worker_response(mut response: Response) -> WorkerResponse {
    let bytes = match http_body::Body::data(response.body_mut()).await {
        None => vec![],
        Some(body_bytes) => match body_bytes {
            Ok(bytes) => bytes.to_vec(),
            Err(_) => vec![]
        },
    };
    let code = response.status().as_u16();

    let mut worker_response = WorkerResponse::from_bytes(bytes).expect("Error parsing Body Bytes");
    worker_response = worker_response.with_status(code);

    let mut headers = Headers::new();
    for (key, value) in response.headers().iter() {
        headers.set(
            key.as_str(),
            value.to_str().expect("failed to convert header value to str"),
        ).unwrap()
    }
    worker_response = worker_response.with_headers(headers);

    worker_response
}


#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Bytes,
        response::{Html},
        response::IntoResponse
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

        let request = to_axum_request(worker_request).await;

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

        let mut request = to_axum_request(worker_request).await;

        let body_bytes: Bytes = http_body::Body::data(request.body_mut()).await.unwrap().unwrap();
        assert_eq!(body_bytes.to_vec(), b"hello world!");
    }

    #[wasm_bindgen_test]
    async fn it_should_convert_the_axum_response_to_a_worker_response() {
        let response = Html::from("Hello World!").into_response();
        let worker_response = to_worker_response(response).await;

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
        let body = http_body::combinators::UnsyncBoxBody::default();
        let response = Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(body)
            .unwrap();


        let worker_response = to_worker_response(response).await;

        assert_eq!(worker_response.status_code(), 200);
        assert_eq!(worker_response.headers().get("Content-Type").unwrap().unwrap(), "text/html");
        let body = match worker_response.body() {
            ResponseBody::Body(body) => body.clone(),
            _ => b"should be empty".to_vec()
        };
        assert_eq!(body.len(), 0);
    }
}
