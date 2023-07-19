use std::ops::Deref;
use std::str::FromStr;
use axum::{
		extract::{Path, State},
		routing::get,
		Router as AxumRouter,
		response::IntoResponse,
};
use axum::http::header::CONTENT_TYPE;
use axum_cloudflare_adapter::{EnvWrapper, to_axum_request, to_worker_response, wasm_compat};
use tower_service::Service;
use worker::{console_log, Env, Request, Response, Date, Result, event, wasm_bindgen_futures, Var};

mod utils;

fn log_request(req: &Request) {
		console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

use url::Url;

#[wasm_compat]
pub async fn index(State(state): State<AxumState>) -> impl IntoResponse {
		let url = Url::from_str("https://logankeenan.com").unwrap();
		let mut response = worker::Fetch::Url(url).send().await.unwrap();
		let body_text = response.text().await.unwrap();

		let env: &Env = state.env_wrapper.env.deref();
		let worker_rs_version: Var = env.var("WORKERS_RS_VERSION").unwrap();

		console_log!("WORKERS_RS_VERSION: {}", worker_rs_version.to_string());

		let content_type = response.headers().get("content-type").unwrap().unwrap();
		axum::response::Response::builder()
				.header(CONTENT_TYPE, content_type)
				.body(body_text)
				.unwrap()
}

#[wasm_compat]
pub async fn with_pathname(Path(path): Path<String>) -> impl IntoResponse {
		let mut url = Url::from_str("https://logankeenan.com").unwrap();
		url.set_path(path.as_str());
		let mut response = worker::Fetch::Url(url).send().await.unwrap();
		let body_text = response.text().await.unwrap();

		let content_type = response.headers().get("content-type").unwrap().unwrap();
		axum::response::Response::builder()
				.header(CONTENT_TYPE, content_type)
				.body(body_text)
				.unwrap()
}

#[derive(Clone)]
pub struct AxumState {
		pub env_wrapper: EnvWrapper,
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
		log_request(&req);
		// Optionally, get more helpful error messages written to the console in the case of a panic.
		utils::set_panic_hook();

		let axum_state = AxumState {
				env_wrapper: EnvWrapper::new(env),
		};

		let mut _router: AxumRouter = AxumRouter::new()
				.route("/", get(index))
				.route("/*path", get(with_pathname))
				.with_state(axum_state);

		let axum_request = to_axum_request(req).await.unwrap();
		let axum_response = _router.call(axum_request).await.unwrap();
		let response = to_worker_response(axum_response).await.unwrap();


		Ok(response)
}
