use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Common derives for most structs
#[proc_macro_attribute]
pub fn common_derives(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #input
    };

    TokenStream::from(expanded)
}

/// For database entities
#[proc_macro_attribute]
pub fn entity_model(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = quote! {
        #[derive(Debug, Clone, Debug, PartialEq, Eq, sea_orm::DeriveEntityModel)]
        #input
    };

    TokenStream::from(expanded)
}

/// For database Enums
#[proc_macro_attribute]
pub fn entity_enum_model(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = quote! {
        #[derive(Debug, Clone, Debug, PartialEq, Eq, sea_orm::EnumIter, sea_orm::DeriveActiveEnum)]
        #input
    };

    TokenStream::from(expanded)
}

/// For API types
#[proc_macro_attribute]
pub fn api_model(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = quote! {
        #[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        #input
    };

    TokenStream::from(expanded)
}
