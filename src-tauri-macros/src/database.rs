use std::{env, fs, path::PathBuf};

use proc_macro::TokenStream;
use quote::quote;

pub fn dlls(_: TokenStream) -> TokenStream {
    let ddls_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("resources")
        .join("ddl");

    let files = fs::read_dir(ddls_dir)
        .unwrap()
        .filter_map(|e| {
            if let Ok(val) = e {
                Some(val.path())
            } else {
                None
            }
        })
        .filter(|p| p.extension().map(|e| e == "sql").unwrap_or(false))
        .collect::<Vec<PathBuf>>();

    let entries = files.iter().map(|path| {
        let name = path.file_name().unwrap().display().to_string();
        let version = name
            .split("_")
            .next()
            .expect("Invalid DDL file name")
            .parse::<u16>()
            .expect("Invalid DDL file name");

        let mut content = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("could not read {}: {}", path.display(), e));

        let description = content
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("--"))
            .unwrap_or_else(|| panic!("missing description comment {}", name))
            .trim()
            .to_string();

        content = content
            .lines()
            .filter_map(|l| {
                let l = l.trim();

                if l.is_empty() || l.starts_with("--") {
                    return None;
                }

                return Some(l.to_string());
            })
            .collect::<Vec<String>>()
            .join("\n");

        quote! {
            DdlVersion {
                version: #version,
                description: #description,
                sql: #content,
            }
        }
    });
    let len = entries.len();

    let expanded = quote! {
        #[derive(Clone, Copy)]
        pub struct DdlVersion {
            pub version: u16,
            pub description: &'static str,
            pub sql: &'static str,
        }

        pub static DDLS: [DdlVersion; #len] = [
            #(#entries),*
        ];
    };

    expanded.into()
}
