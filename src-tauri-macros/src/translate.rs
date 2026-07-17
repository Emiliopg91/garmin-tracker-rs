use proc_macro::TokenStream;
use quote::quote;
use std::{collections::BTreeMap, fs, sync::LazyLock};
use syn::{Expr, Lit, Token, parse::Parser, punctuated::Punctuated};

pub static TRANSLATIONS: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    let path = std::env::var("TRANSLATIONS_YAML").unwrap();
    let content = fs::read_to_string(path).unwrap();

    let translations: BTreeMap<String, String> =
        serde_yaml::from_str(&content.to_string()).unwrap();

    translations
});

pub fn translate(input: TokenStream) -> TokenStream {
    let mut args = Punctuated::<Expr, Token![,]>::parse_terminated
        .parse(input)
        .unwrap()
        .into_iter();

    let first = args.next().expect("Missing translation key");
    let key = match first {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(s) => s.value(),
            _ => panic!("First argument must be a string literal"),
        },
        _ => panic!("First argument must be a string literal"),
    };
    let params: Vec<_> = args.collect();
    let translation = (*TRANSLATIONS)
        .get(&key)
        .expect("Missing translation entry");

    let expand = if params.is_empty() {
        quote! {
            #translation.to_string()
        }
    } else {
        quote! {
            format!(#translation, #( #params ),*)
        }
    };

    expand.into()
}
