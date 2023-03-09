use worker::{
		Result, Response, Request,
		Env,
		event,
		console_log,
		Date,
};

use axum::{
		response::{Html},
		routing::get,
		Router as AxumRouter,
};
use axum_cloudflare_adapter::{to_axum_request, to_worker_response};
use tower_service::Service;

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

async fn index() -> Html<&'static str> {
		let url1 = url::Url::parse("https://logankeenan.com").unwrap();
		let fetch1 = worker::Fetch::Url(url1);
		// Errors out when.await exists
		let response = fetch1.send().await;

		Html("<p>Hello from Axum!</p>")
}

#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: worker::Context) -> Result<Response> {
		log_request(&req);


		// Optionally, get more helpful error messages written to the console in the case of a panic.
		utils::set_panic_hook();

		let router = get(index);
		let mut _router: AxumRouter = AxumRouter::new()
				.route("/", router);

		let axum_request = to_axum_request(req).await.unwrap();
		let axum_response = _router.call(axum_request).await.unwrap();
		let response = to_worker_response(axum_response).await.unwrap();


		Ok(response)
}
