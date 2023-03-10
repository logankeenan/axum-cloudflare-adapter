# axum-cloudflare-adapter

[![Crates.io](https://img.shields.io/crates/v/axum-cloudflare-adapter)](https://crates.io/crates/axum-cloudflare-adapter)

An adapter to easily allow an [Axum](https://github.com/tokio-rs/axum) server to be run within a Cloudflare worker.

## Usage
```rust
use worker::*;

use axum::{
    response::{Html},
    routing::get,
    Router as AxumRouter,
};
use axum_cloudflare_adapter::{to_axum_request, to_worker_response};
use tower_service::Service;

async fn index() -> Html<&'static str> {
		Html("<p>Hello from Axum!</p>")
}

#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: worker::Context) -> Result<Response> {
    let mut _router: AxumRouter = AxumRouter::new()
            .route("/", get(index));
    
    let axum_request = to_axum_request(req).await.unwrap();
    let axum_response = _router.call(axum_request).await.unwrap();
    let response = to_worker_response(axum_response).await.unwrap();
    
    Ok(response)
}
```

## Running tests
`cd adapter && wasm-pack test --firefox --headless`

## Building
`cargo build --target wasm32-unknown-unknown`

## Example
The `/example` directory contains a Cloudflare worker running an Axum sever