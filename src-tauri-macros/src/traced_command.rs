use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, LitBool, Pat, ReturnType, Type, parse::Parser, parse_macro_input};

/// Parsed `#[traced_command(...)]` attribute options.
struct Args {
    log_payload: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self { log_payload: true }
    }
}

fn parse_traced_command_attrs(attrs: TokenStream) -> syn::Result<Args> {
    let mut args = Args::default();

    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("log_payload") {
            let lit = meta.value()?.parse::<LitBool>()?;
            args.log_payload = lit.value();
        } 
        Ok(())
    });
    parser.parse(attrs)?;

    Ok(args)
}

/// Generates the traced wrapper body for the annotated command function.
pub fn traced_command(attrs: TokenStream, item: TokenStream) -> TokenStream {
    macro_rules! bail_on_err {
        ($expr:expr) => {
            match $expr {
                Ok(value) => value,
                Err(err) => return err.to_compile_error().into(),
            }
        };
    }

    let args = bail_on_err!(parse_traced_command_attrs(attrs));
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
                    if last_segment_is(&pat_type.ty, "WebviewWindow")
                        || last_segment_is(&pat_type.ty, "AppHandle")
                        || last_segment_is(&pat_type.ty, "State")
                    {
                        return None;
                    }
                    Some(pat_ident.ident.clone())
                }
                _ => None,
            },
        })
        .collect();

    let param_keys: Vec<_> = param_names.iter().map(|p| p.to_string()).collect();
    let output_ty = match &sig.output {
        syn::ReturnType::Default => quote! { () },
        syn::ReturnType::Type(_, ty) => quote! { #ty },
    };

    let call = if is_async {
        quote! { (async move || -> #output_ty #block)().await }
    } else {
        quote! { (move || -> #output_ty #block)() }
    };

    let result_json_code = if last_segment_is_return(&sig.output, "Result") {
        quote! {
            let __result_json = match &result {
                Ok(v) => serde_json::json!(v),
                Err(e) => serde_json::json!({ "error": e.to_string() }),
            };
        }
    } else if last_segment_is_return(&sig.output, "Option") {
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
    };

    let (invoke_log_code, finish_log_code) = if args.log_payload {
        (
            quote! {
                let __params_json = serde_json::json!({
                    #( #param_keys: #param_names ),*
                });

                tauri_plugin_log::log::debug!(
                    "Invoking command '{}' with params {}",
                    #name,
                    __params_json.to_string()
                );
            },
            quote! {
                #result_json_code
                let json_str = __result_json.to_string();

                tauri_plugin_log::log::debug!(
                    "Finished command '{}' after {:.3} with response {}",
                    #name,
                    t0.elapsed().as_secs_f64(),
                    json_str
                );
            },
        )
    } else {
        (
            quote! {
                let __params_json = serde_json::json!({
                    #( #param_keys: #param_names ),*
                });

                tauri_plugin_log::log::debug!(
                    "Invoking command '{}' with payload of {} bytes",
                    #name,
                    __params_json.to_string().len()
                );
            },
            quote! {
                #result_json_code
                let __response_size = __result_json.to_string().len();

                tauri_plugin_log::log::debug!(
                    "Finished command '{}' after {:.3} with response of {} bytes",
                    #name,
                    t0.elapsed().as_secs_f64(),
                    __response_size
                );
            },
        )
    };

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            let __debug_enabled = tauri_plugin_log::log::log_enabled!(tauri_plugin_log::log::Level::Debug);
            let t0 = std::time::Instant::now();

            if __debug_enabled {
                #invoke_log_code
            }

            let result: #output_ty = #call;

            if __debug_enabled {
                #finish_log_code
            }

            result
        }
    };

    expanded.into()
}

/// Checks whether a type's last path segment matches `name` (e.g. `Result`, `Option`, `WebviewWindow`).
fn last_segment_is(ty: &Type, name: &str) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == name;
    }
    false
}

/// Same as `last_segment_is`, but applied to a function's return type.
fn last_segment_is_return(output: &ReturnType, name: &str) -> bool {
    if let ReturnType::Type(_, ty) = output {
        last_segment_is(ty, name)
    } else {
        false
    }
}
