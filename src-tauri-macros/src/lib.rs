use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn traced_command(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;
    let attrs = &input_fn.attrs;
    let name = &sig.ident.to_string();

    let is_async = sig.asyncness.is_some();

    let call = if is_async {
        quote! { (async move || #block)().await }
    } else {
        quote! { (|| #block)() }
    };

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            let t0 = std::time::Instant::now();
            tauri_plugin_log::log::debug!("Invoking command {}...", #name);
            let result = #call;
            tauri_plugin_log::log::debug!("Command {} finished after {:.3}s", #name, t0.elapsed().as_secs_f64());
            result
        }
    };

    expanded.into()
}
