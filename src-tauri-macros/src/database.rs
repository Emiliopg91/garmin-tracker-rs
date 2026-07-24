use std::{env, fs, path::PathBuf};

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input, parse_quote};

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

                Some(l.to_string())
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

struct FieldInfo {
    ident: syn::Ident,
    column: String,
    ty: syn::Type,
    const_ident: syn::Ident,
    is_id: bool,
}

pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let mut table_name = struct_name.to_string().to_lowercase();
    for attr in &input.attrs {
        if !attr.path().is_ident("entity") {
            continue;
        }

        let mut found_table = false;
        let result = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("table") {
                let lit: syn::LitStr = meta.value()?.parse()?;
                table_name = lit.value().trim().to_string();
                if table_name.is_empty() {
                    return Err(meta.error("Attribute table cannot be empty"));
                }
                found_table = true;
                Ok(())
            } else {
                Err(meta.error("Attribute `entity` not recognized, expected `table = \"...\"`"))
            }
        });

        if let Err(err) = result {
            return err.to_compile_error().into();
        }
    }

    let named_fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "Entity only can be derived in named structs",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "Entity only can be derived in structs")
                .to_compile_error()
                .into();
        }
    };

    let mut has_no_column = false;
    let mut fields: Vec<FieldInfo> = Vec::new();

    for f in named_fields.iter() {
        let no_column = f.attrs.iter().any(|attr| attr.path().is_ident("no_column"));
        if no_column {
            has_no_column = true;
            continue;
        }

        let ident = f.ident.clone().unwrap();
        let name = ident.to_string();
        let const_ident = format_ident!(
            "{}_COLUMN_{}",
            &struct_name.to_string().to_uppercase(),
            name.to_uppercase()
        );
        let is_id = f.attrs.iter().any(|attr| attr.path().is_ident("id"));
        let mut column_name = name.to_lowercase();

        for attr in &f.attrs {
            if attr.path().is_ident("column") {
                let result = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("name") {
                        let lit: syn::LitStr = meta.value()?.parse()?;
                        column_name = lit.value().trim().to_string();
                        if column_name.is_empty() {
                            return Err(meta.error("Attribute name cannot be empty"));
                        }
                        Ok(())
                    } else {
                        Err(meta
                            .error("Attribute `column` not recognized, expected `name = \"...\"`"))
                    }
                });

                if let Err(err) = result {
                    return err.to_compile_error().into();
                }
            }
        }

        fields.push(FieldInfo {
            ident,
            column: column_name,
            ty: f.ty.clone(),
            const_ident,
            is_id,
        });
    }

    let field_constants = fields.iter().map(|f| {
        let const_ident = &f.const_ident;
        let name = &f.column;
        quote! {
            pub const #const_ident: crate::garmin::database::dao::helpers::types::column_name::ColumnName =
                crate::garmin::database::dao::helpers::types::column_name::ColumnName::new(#name);
        }
    });

    let field_name_list = fields.iter().map(|f| &f.const_ident);

    let map_from_rows_lines = fields.iter().enumerate().map(|(idx, f)| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote! {
            #ident: row.get::<_, #ty>(#idx)?
        }
    });

    let default_spread = if has_no_column {
        quote! { , ..Default::default() }
    } else {
        quote! {}
    };

    let get_values_lines = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! { self.#ident.clone().into() }
    });

    let id_fields: Vec<&FieldInfo> = fields.iter().filter(|f| f.is_id).collect();
    let instance_operations = if id_fields.is_empty() {
        eprintln!(
            "\x1b[33m{} entity has no primary key defined, select, update and delete operations by ID will not be available\x1b[0m",
            struct_name
        );
        quote! {}
    } else {
        let by_id_params: Vec<TokenStream2> = id_fields
            .iter()
            .map(|f| {
                let ident = &f.ident;
                let mut ty = &f.ty;
                let str_ty: Type = parse_quote!(&str);

                if let Type::Path(type_path) = ty
                    && let Some(segment) = type_path.path.segments.last()
                    && segment.ident == "String"
                {
                    ty = &str_ty;
                }
                quote! { #ident: #ty }
            })
            .collect();

        fn build_condition(
            id_fields: &[&FieldInfo],
            value_for: impl Fn(&FieldInfo) -> TokenStream2,
        ) -> TokenStream2 {
            if id_fields.len() > 1 {
                let cond = id_fields.iter().map(|f| {
                    let const_ident = &f.const_ident;
                    let value = value_for(f);
                    quote! {
                        crate::garmin::database::dao::helpers::types::where_clause::Where::Eq(#const_ident, #value)
                    }
                });
                quote! {
                    crate::garmin::database::dao::helpers::types::where_clause::Where::And(vec![
                        #(#cond),*
                    ])
                }
            } else {
                let f = id_fields[0];
                let const_ident = &f.const_ident;
                let value = value_for(f);
                quote! {
                    crate::garmin::database::dao::helpers::types::where_clause::Where::Eq(
                        #const_ident, #value
                    )
                }
            }
        }

        let id_condition = build_condition(&id_fields, |f| {
            let ident = &f.ident;
            quote! { #ident.into() }
        });

        let update_delete_condition = build_condition(&id_fields, |f| {
            let ident = &f.ident;
            quote! { self.#ident.clone().into() }
        });

        let update_sets: Vec<TokenStream2> = fields
            .iter()
            .filter(|f| !f.is_id)
            .map(|f| {
                let ident = &f.ident;
                let const_ident = &f.const_ident;
                quote! {
                    .set(#const_ident, self.#ident.clone().into())
                }
            })
            .collect();

        quote! {
            impl #struct_name {
                pub fn select_by_id(
                    #(#by_id_params),*
                ) -> crate::garmin::database::errors::Result<Option<Self>> {
                    Ok(<#struct_name as crate::garmin::database::dao::Entity>::select()
                        .where_(#id_condition)
                        .fetch()?
                        .into_iter()
                        .next())
                }

                pub fn select_by_id_in_tx(
                    tx: &rusqlite::Transaction,
                    #(#by_id_params),*
                ) -> crate::garmin::database::errors::Result<Option<Self>> {
                    Ok(<#struct_name as crate::garmin::database::dao::Entity>::select()
                        .where_(#id_condition)
                        .fetch_in_tx(tx)?
                        .into_iter()
                        .next())
                }

                pub fn update_by_id(&self) -> crate::garmin::database::errors::Result<()> {
                    <#struct_name as crate::garmin::database::dao::Entity>::update()
                        #(#update_sets)*
                        .where_(#update_delete_condition)
                        .execute()
                }

                pub fn update_by_id_in_tx(
                    &self,
                    tx: &rusqlite::Transaction,
                ) -> crate::garmin::database::errors::Result<()> {
                    <#struct_name as crate::garmin::database::dao::Entity>::update()
                        #(#update_sets)*
                        .where_(#update_delete_condition)
                        .execute_in_tx(tx)
                }

                pub fn delete_by_id(&self) -> crate::garmin::database::errors::Result<()> {
                    <#struct_name as crate::garmin::database::dao::Entity>::delete()
                        .where_(#update_delete_condition)
                        .execute()
                }

                pub fn delete_by_id_in_tx(
                    &self,
                    tx: &rusqlite::Transaction,
                ) -> crate::garmin::database::errors::Result<()> {
                    <#struct_name as crate::garmin::database::dao::Entity>::delete()
                        .where_(#update_delete_condition)
                        .execute_in_tx(tx)
                }
            }
        }
    };

    let expanded = quote! {
        #(#field_constants)*

        impl crate::garmin::database::dao::Entity for #struct_name {
            const TABLE_NAME: &'static str = #table_name;
            const FIELDS: &'static [crate::garmin::database::dao::helpers::types::column_name::ColumnName] =
                &[ #(#field_name_list),* ];

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

        #instance_operations
    };

    expanded.into()
}
