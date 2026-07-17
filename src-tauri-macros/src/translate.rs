use proc_macro::TokenStream;
use quote::quote;
use std::{collections::BTreeMap, path::PathBuf, sync::LazyLock};
use syn::{parse::Parser, punctuated::Punctuated, Expr, Lit, Token};

pub static TRANSLATIONS: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    let translations_file = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("resources")
        .join("translations.yaml");

    let content = std::fs::read_to_string(translations_file).unwrap();
    let translations_map: BTreeMap<String, BTreeMap<String, String>> =
        serde_yaml::from_str(content.as_str()).unwrap();

    let default_lang = "en";

    let mut lang_var = std::env::var("LANG").unwrap_or("C".to_string());
    if lang_var == "C" {
        lang_var = "en".into();
    }
    if lang_var.contains(".") {
        lang_var = lang_var.split(".").next().unwrap().into();
    }
    if lang_var.contains("_") {
        lang_var = lang_var.split("_").next().unwrap().into();
    }
    lang_var = lang_var.to_lowercase();

    let mut translations_filtered = BTreeMap::new();
    for (key, values) in &translations_map {
        translations_filtered.insert(
            key.clone(),
            values
                .get(&lang_var)
                .unwrap_or(values.get(default_lang).unwrap_or(&key.to_string()))
                .clone(),
        );
    }

    translations_filtered
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
