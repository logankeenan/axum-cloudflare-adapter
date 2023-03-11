use std::str::FromStr;
use axum::{
		response::{Html},
		routing::get,
		Router as AxumRouter,
};
use axum::extract::Path;
use axum::response::IntoResponse;
use axum_cloudflare_adapter::{
		to_axum_request,
		to_worker_response,
		worker_route_compat,
};
use tower_service::Service;
use worker::{console_log, Env, Request, Response, Date, Result, event, wasm_bindgen_futures};

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

#[worker_route_compat]
pub async fn index() -> impl IntoResponse {
		let url = Url::from_str("https://logankeenan.com").unwrap();
		let mut response = worker::Fetch::Url(url).send().await.unwrap();
		let body_text = response.text().await.unwrap();
		Html(body_text)
}

#[worker_route_compat]
pub async fn with_pathname(Path(path): Path<String>) -> impl IntoResponse {
		let mut url = Url::from_str("https://logankeenan.com").unwrap();
		url.set_path(path.as_str());
		let mut response = worker::Fetch::Url(url).send().await.unwrap();
		let body_text = response.text().await.unwrap();
		Html(body_text)
}


#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: worker::Context) -> Result<Response> {
		log_request(&req);

		// Optionally, get more helpful error messages written to the console in the case of a panic.
		utils::set_panic_hook();

		let mut _router: AxumRouter = AxumRouter::new()
				.route("/", get(index))
				.route("/*path", get(with_pathname));

		let axum_request = to_axum_request(req).await.unwrap();
		let axum_response = _router.call(axum_request).await.unwrap();
		let response = to_worker_response(axum_response).await.unwrap();


		Ok(response)
}
