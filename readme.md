# axum-cloudflare-adapter

An adapter to easily allow an [Axum](https://github.com/tokio-rs/axum) server to be run within a Cloudflare worker.

## Running tests
`wasm-pack test --firefox --headless`

## Building
`cargo build --target wasm32-unknown-unknown`

## Example
The `/example` directory contains a Cloudflare worker running an Axum sever