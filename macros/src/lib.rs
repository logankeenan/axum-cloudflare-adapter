//! Axum Cloudflare Adapter Macros
//!
//! Axum handlers return a `Send` future. However, JS types do not return a `Send`
//! future. `worker_route_compat` will provide compatability between the return types.
//! See [axum-cloudflare-adapter](https://github.com/logankeenan/axum-cloudflare-adapter) for usage.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn worker_route_compat(_attr: TokenStream, stream: TokenStream) -> TokenStream {
    let stream_clone = stream.clone();
    let input = parse_macro_input!(stream_clone as ItemFn);

    let ItemFn { attrs, vis, sig, block } = input;
    let stmts = &block.stmts;
    let result = quote! {
        #(#attrs)* #vis #sig {
            let (tx, rx) = oneshot::channel();
		    wasm_bindgen_futures::spawn_local(async move {
				let result = {
                    #(#stmts)*
                };
				tx.send(result).unwrap();
		    });

		    rx.await.unwrap()
        }
    };

    TokenStream::from(result)
}

