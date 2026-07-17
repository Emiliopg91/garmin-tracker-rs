use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{FnArg, ItemFn, Pat, ReturnType, Type, parse_macro_input};

pub fn traced_command(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;
    let attrs = &input_fn.attrs;
    let name = sig.ident.to_string();

    let is_async = sig.asyncness.is_some();

    let param_names: Vec<_> = sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => match &*pat_type.pat {
                Pat::Ident(pat_ident) => {
                    let type_name = pat_type.ty.to_token_stream().to_string();
                    if type_name == "WebviewWindow" || type_name == "AppHandle" {
                        return None;
                    }
                    Some(pat_ident.ident.clone())
                }
                _ => None,
            },
        })
        .collect();

    let param_keys: Vec<_> = param_names.iter().map(|p| p.to_string()).collect();

    let call = if is_async {
        quote! { (async move || #block)().await }
    } else {
        quote! { (|| #block)() }
    };

    let returns_result = is_result_type(&sig.output);
    let result_json_code = if returns_result {
        quote! {
            let __result_json = match &result {
                Ok(v) => serde_json::json!(v),
                Err(e) => serde_json::json!({ "error": e.to_string() }),
            };
        }
    } else {
        let returns_option = is_option_type(&sig.output);
        if returns_option {
            quote! {
                let __result_json = match &result {
                    Some(v) => serde_json::json!(v),
                    None => serde_json::Value::Null,
                };
            }
        } else {
            quote! {
                let __result_json = serde_json::json!(&result);
            }
        }
    };

    let expanded = quote! {
        #(#attrs)*
        #[cfg(debug_assertions)]
        #vis #sig {
            let t0 = std::time::Instant::now();

            let __params_json = serde_json::json!({
                #( #param_keys: #param_names ),*
            });

            tauri_plugin_log::log::debug!(
                "Invoking command '{}' with params {}",
                #name,
                __params_json.to_string()
            );

            let result = #call;

            #result_json_code
            let json_str = __result_json.to_string();

            tauri_plugin_log::log::debug!(
                "Finished command '{}' after {:.3} with response {}",
                #name,
                t0.elapsed().as_secs_f64(),
                json_str
            );

            result
        }

        #(#attrs)*
        #[cfg(not(debug_assertions))]
        #vis #sig #block
    };

    expanded.into()
}

fn is_result_type(output: &ReturnType) -> bool {
    if let ReturnType::Type(_, ty) = output {
        if let Type::Path(type_path) = &**ty {
            if let Some(segment) = type_path.path.segments.last() {
                return segment.ident == "Result";
            }
        }
    }
    false
}

fn is_option_type(output: &ReturnType) -> bool {
    if let ReturnType::Type(_, ty) = output {
        if let Type::Path(type_path) = &**ty {
            if let Some(segment) = type_path.path.segments.last() {
                return segment.ident == "Option";
            }
        }
    }
    false
}
