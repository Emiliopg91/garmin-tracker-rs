use std::{env, fs, path::PathBuf};

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput};

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

pub fn derive_entity(input: TokenStream) -> TokenStream {
    let mut has_no_fields = false;
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let table_name = struct_name.to_string().to_uppercase();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(named) => &named.named,
            _ => panic!("Derive only available with named types"),
        },
        _ => panic!("Derive only available for structs"),
    };

    let mut id_field_names = Vec::new();
    let mut id_field_idents = Vec::new();
    let mut id_field_types = Vec::new();
    let mut field_names = Vec::new();
    let mut field_idents = Vec::new();
    let mut field_types = Vec::new();
    let mut field_constants = Vec::new();

    fields.iter().for_each(|f| {
        let mut no_field = false;

        for attr in &f.attrs {
            if attr.path().is_ident("no_field") {
                no_field = true;
                has_no_fields = true;
            }
        }

        if !no_field {
            let name = f.ident.clone().unwrap().to_string();
            let const_ident = format_ident!("{}_COLUMN_{}", table_name, name.to_uppercase());
            field_constants.push(quote! {
                pub const #const_ident: crate::garmin::database::dao::helpers::types::column_name::ColumnName = #name;
            });

            for attr in &f.attrs {
                if attr.path().is_ident("id") {
                    id_field_names.push(const_ident.clone());
                    id_field_idents.push(f.ident.clone().unwrap());
                    id_field_types.push(f.ty.clone());
                }
            }

            field_names.push(const_ident);
            field_idents.push(f.ident.clone().unwrap());
            field_types.push(f.ty.clone());
        }
    });

    let map_from_rows_lines = field_idents
        .iter()
        .zip(field_names.iter())
        .zip(field_types.iter())
        .map(|((name, column), typ)| {
            quote! {
                #name: row.get::<_, #typ>(#column)?
            }
        });

    let default_spread = if has_no_fields {
        quote! { , ..Default::default() }
    } else {
        quote! {}
    };

    let get_values_lines = field_idents.iter().map(|ident| {
        quote! {
            self.#ident.clone().into()
        }
    });

    let by_id_params = id_field_idents
        .iter()
        .zip(id_field_types.iter())
        .map(|(name, ty)| {
            quote! {
                #name: #ty
            }
        });

    let id_condition = if id_field_names.len() > 1 {
        let cond = id_field_idents.iter().map(|ident| {
            let const_ident = format_ident!("{}_COLUMN_{}", table_name, ident.clone().to_string().to_uppercase());
            quote!{
                crate::garmin::database::dao::helpers::types::where_clause::Where::Eq(#const_ident, #ident.into())
            }
        });
        quote! {
            crate::garmin::database::dao::helpers::types::where_clause::Where::And(vec![
                #(#cond),*
            ])
        }
    } else {
        let name = id_field_idents.get(0).unwrap().clone();
        let const_ident =
            format_ident!("{}_COLUMN_{}", table_name, name.to_string().to_uppercase());
        quote! {
            crate::garmin::database::dao::helpers::types::where_clause::Where::Eq(
                #const_ident, #name.into()
            )
        }
    };

    let expanded = quote! {
        #(#field_constants)*
        impl crate::garmin::database::dao::Entity for #struct_name {
            const TABLE_NAME: &'static str = #table_name;
            const FIELDS: &'static [crate::garmin::database::dao::helpers::types::column_name::ColumnName] = &[ #(#field_names),* ];

            fn map_from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
                Ok(Self {
                    #(#map_from_rows_lines),*
                    #default_spread
                })
            }

            fn get_values(&self) -> Vec<crate::garmin::database::dao::helpers::types::value::Value> {
                vec![
                    #(#get_values_lines),*
                ]
            }
        }

        impl #struct_name {
            pub fn select_by_id(
                #(#by_id_params),*
            ) -> crate::garmin::database::errors::Result<Option<Self>>{
                Ok(crate::garmin::database::dao::Entity::select()
                .where_(
                    #id_condition
                )
                .limit(1)
                .fetch()?
                .into_iter()
                .next())
            }
        }
    };

    expanded.into()
}
