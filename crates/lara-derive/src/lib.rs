extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Lit, Meta,
    punctuated::Punctuated, Token,
};

// ── #[derive(Model)] ────────────────────────────────────────────────────────

/// Derives `lara_db::Model` for a struct.
///
/// ```rust
/// #[derive(Debug, Clone, Serialize, Deserialize, Model)]
/// #[lara(table = "users", primary_key = "id")]
/// pub struct User {
///     pub id: Option<i64>,
///     pub name: String,
///     pub email: String,
///     #[lara(hidden)]
///     pub password: String,
///     #[lara(fillable)]
///     pub bio: Option<String>,
/// }
/// ```
#[proc_macro_derive(Model, attributes(lara))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Parse struct-level #[lara(...)] attributes
    let mut table_name = to_table_name(&name.to_string());
    let mut primary_key = "id".to_string();
    let mut timestamps = true;
    let mut soft_deletes = false;

    for attr in &input.attrs {
        if !attr.path().is_ident("lara") {
            continue;
        }
        if let Ok(list) = attr.parse_args_with(
            Punctuated::<Meta, Token![,]>::parse_terminated,
        ) {
            for meta in list {
                match &meta {
                    Meta::NameValue(nv) if nv.path.is_ident("table") => {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(s), ..
                        }) = &nv.value
                        {
                            table_name = s.value();
                        }
                    }
                    Meta::NameValue(nv) if nv.path.is_ident("primary_key") => {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(s), ..
                        }) = &nv.value
                        {
                            primary_key = s.value();
                        }
                    }
                    Meta::Path(p) if p.is_ident("timestamps") => timestamps = true,
                    Meta::Path(p) if p.is_ident("no_timestamps") => timestamps = false,
                    Meta::Path(p) if p.is_ident("soft_deletes") => soft_deletes = true,
                    _ => {}
                }
            }
        }
    }

    // Collect field-level #[lara(hidden)] and #[lara(fillable)] markers
    let mut hidden_fields: Vec<String> = Vec::new();
    let mut fillable_fields: Vec<String> = Vec::new();

    if let Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            for field in &fields.named {
                let field_name = field.ident.as_ref().unwrap().to_string();
                for attr in &field.attrs {
                    if !attr.path().is_ident("lara") {
                        continue;
                    }
                    if let Ok(list) = attr.parse_args_with(
                        Punctuated::<Meta, Token![,]>::parse_terminated,
                    ) {
                        for meta in &list {
                            if let Meta::Path(p) = meta {
                                if p.is_ident("hidden") {
                                    hidden_fields.push(field_name.clone());
                                }
                                if p.is_ident("fillable") {
                                    fillable_fields.push(field_name.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let table_name_lit = &table_name;
    let primary_key_lit = &primary_key;
    let hidden_strs: Vec<_> = hidden_fields.iter().map(|s| s.as_str()).collect();
    let fillable_strs: Vec<_> = fillable_fields.iter().map(|s| s.as_str()).collect();

    let expanded = quote! {
        impl lara_db::model::ModelMeta for #name {
            fn table_name() -> &'static str {
                #table_name_lit
            }

            fn primary_key_column() -> &'static str {
                #primary_key_lit
            }

            fn hidden_columns() -> &'static [&'static str] {
                &[#(#hidden_strs),*]
            }

            fn fillable_columns() -> &'static [&'static str] {
                &[#(#fillable_strs),*]
            }

            fn with_timestamps() -> bool {
                #timestamps
            }

            fn with_soft_deletes() -> bool {
                #soft_deletes
            }
        }
    };

    TokenStream::from(expanded)
}

// ── #[derive(Command)] ──────────────────────────────────────────────────────

/// Marks a struct as a Lara console command (stub — registers the `name` / `description`).
#[proc_macro_derive(Command, attributes(lara))]
pub fn derive_command(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut cmd_name = to_kebab(&name.to_string());
    let mut cmd_desc = String::new();

    for attr in &input.attrs {
        if !attr.path().is_ident("lara") {
            continue;
        }
        if let Ok(list) = attr.parse_args_with(
            Punctuated::<Meta, Token![,]>::parse_terminated,
        ) {
            for meta in list {
                if let Meta::NameValue(nv) = &meta {
                    if nv.path.is_ident("name") {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(s), ..
                        }) = &nv.value
                        {
                            cmd_name = s.value();
                        }
                    } else if nv.path.is_ident("description") {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(s), ..
                        }) = &nv.value
                        {
                            cmd_desc = s.value();
                        }
                    }
                }
            }
        }
    }

    let expanded = quote! {
        impl lara_console::CommandMeta for #name {
            fn command_name() -> &'static str { #cmd_name }
            fn command_description() -> &'static str { #cmd_desc }
        }
    };

    TokenStream::from(expanded)
}

// ── #[derive(Job)] ──────────────────────────────────────────────────────────

#[proc_macro_derive(Job, attributes(lara))]
pub fn derive_job(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut queue_name = "default".to_string();
    let mut max_tries: u32 = 3;
    let mut timeout_secs: u64 = 60;

    for attr in &input.attrs {
        if !attr.path().is_ident("lara") {
            continue;
        }
        if let Ok(list) = attr.parse_args_with(
            Punctuated::<Meta, Token![,]>::parse_terminated,
        ) {
            for meta in list {
                if let Meta::NameValue(nv) = &meta {
                    match &nv.value {
                        syn::Expr::Lit(syn::ExprLit { lit: Lit::Str(s), .. })
                            if nv.path.is_ident("queue") =>
                        {
                            queue_name = s.value();
                        }
                        syn::Expr::Lit(syn::ExprLit { lit: Lit::Int(n), .. })
                            if nv.path.is_ident("tries") =>
                        {
                            max_tries = n.base10_parse().unwrap_or(3);
                        }
                        syn::Expr::Lit(syn::ExprLit { lit: Lit::Int(n), .. })
                            if nv.path.is_ident("timeout") =>
                        {
                            timeout_secs = n.base10_parse().unwrap_or(60);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let expanded = quote! {
        impl lara_queue::JobMeta for #name {
            fn queue_name() -> &'static str { #queue_name }
            fn max_tries() -> u32 { #max_tries }
            fn timeout_secs() -> u64 { #timeout_secs }
        }
    };

    TokenStream::from(expanded)
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn to_table_name(struct_name: &str) -> String {
    // PascalCase → snake_case plural  (User → users, UserRole → user_roles)
    let snake = to_snake(struct_name);
    format!("{}s", snake)
}

fn to_snake(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_lowercase().next().unwrap());
    }
    out
}

fn to_kebab(s: &str) -> String {
    to_snake(s).replace('_', "-")
}
