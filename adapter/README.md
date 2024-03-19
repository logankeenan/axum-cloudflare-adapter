# axum-cloudflare-adapter

[![Crates.io](https://img.shields.io/crates/v/axum-cloudflare-adapter)](https://crates.io/crates/axum-cloudflare-adapter)

An adapter to easily run an [Axum](https://github.com/tokio-rs/axum) server in a Cloudflare worker.

## Cloudflare workers perliminary native support for Axum on 0.0.21+

Axum support in workers-rs is enabled by the [http](https://github.com/cloudflare/workers-rs?tab=readme-ov-file#http-feature) feature in worker-rs.

This is possible because both Axum and worker-rs http uses the same [http](https://docs.rs/http/latest/http/) crate.

This adapter can be used as an easy way to migrate from the non http workers-rs version to the http version:
1. Do not change your current workers-rs project dependency on the non http version of workers-rs (keep the http flag disabled).
1. Add the dependency to this adapter.
2. Add a catch all route to the existing router:
```rust
    .or_else_any_method_async("/*catchall", |_, ctx| async move {
```
3. Inside the catch all route, add an axum router like in the example bellow.
4. Start to incrementally migrate the paths one by one, from the old router to the axum router.
5. Once finished, drop the dependency on this adapter and enable the "http" flag on workers-rs.
6. If you have any issues you can ask for help on #rust-on-workers on discord or open an issue in workers-rs github.

## Usage

```rust
use worker::*;
use axum::{
    response::{Html},
    routing::get,
    Router as AxumRouter,
    extract::State,
};
use axum_cloudflare_adapter::{to_axum_request, to_worker_response, wasm_compat, EnvWrapper};
use tower_service::Service;
use std::ops::Deref;

#[derive(Clone)]
pub struct AxumState {
    pub env_wrapper: EnvWrapper,
}

#[wasm_compat]
async fn index(State(state): State<AxumState>) -> Html<&'static str> {
    let env: &Env = state.env_wrapper.env.deref();
    let worker_rs_version: Var = env.var("WORKERS_RS_VERSION").unwrap();
    console_log!("WORKERS_RS_VERSION: {}", worker_rs_version.to_string());
    Html("<p>Hello from Axum!</p>")
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    let mut _router: AxumRouter = AxumRouter::new()
        .route("/", get(index))
        .with_state(AxumState {
            env_wrapper: EnvWrapper::new(env),
        });
    let axum_request = to_axum_request(req).await.unwrap();
    let axum_response = _router.call(axum_request).await.unwrap();
    let response = to_worker_response(axum_response).await.unwrap();
    Ok(response)
}
```

## Running tests

`cd adapter && wasm-pack test --firefox --headless`

## Building

`cd adapter && cargo build --target wasm32-unknown-unknown`

## Example

The `/example` directory contains a Cloudflare worker running an Axum sever
