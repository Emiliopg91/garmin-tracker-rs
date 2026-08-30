use proc_macro::TokenStream;
use quote::quote;
use std::{collections::BTreeMap, fs, sync::LazyLock};
use syn::{Expr, Lit, Token, parse::Parser, punctuated::Punctuated, spanned::Spanned};

pub static TRANSLATIONS: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    let path = std::env::var("TRANSLATIONS_YAML").unwrap();
    let content = fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&content).unwrap()
});

/// Entry point for the `translate!` macro: converts a `translate_impl` error into a compile error.
pub fn translate(input: TokenStream) -> TokenStream {
    match translate_impl(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Parses `translate!("key", args...)`, looks up `"key"` in `TRANSLATIONS`, and expands to a `String`/`format!` call.
fn translate_impl(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let mut args = Punctuated::<Expr, Token![,]>::parse_terminated
        .parse(input)?
        .into_iter();

    let first = args.next().ok_or_else(|| {
        syn::Error::new(proc_macro2::Span::call_site(), "missing translation key")
    })?;

    let key_lit = match &first {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(s) => s.clone(),
            _ => {
                return Err(syn::Error::new(
                    first.span(),
                    "first argument must be a string literal",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new(
                first.span(),
                "first argument must be a string literal",
            ));
        }
    };

    let key = key_lit.value();
    let params: Vec<_> = args.collect();

    let translation = TRANSLATIONS.get(&key).ok_or_else(|| {
        syn::Error::new(
            key_lit.span(),
            format!("missing translation entry for \"{}\"", key),
        )
    })?;

    let ph_num = translation.matches("{}").count();
    if ph_num != params.len() {
        return Err(syn::Error::new(
            key_lit.span(),
            format!(
                "\"{}\" differs in length with replacements: {} - {}",
                translation,
                ph_num,
                params.len()
            ),
        ));
    }

    let expand = if params.is_empty() {
        quote! { #translation.to_string() }
    } else {
        quote! { format!(#translation, #( #params ),*) }
    };

    Ok(expand)
}
