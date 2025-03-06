use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

fn filter_attributes(attrs: &[syn::Attribute], exclude: &[&str]) -> Vec<syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| !exclude.iter().any(|&name| attr.path().is_ident(name)))
        .cloned()
        .collect() // Return the filtered list of attributes
}

#[proc_macro_attribute]
pub fn source(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("Source: {:?} - {:?}", attr, item);
    TokenStream::from(quote! {})
}

#[proc_macro_derive(FilterMetaData)]
pub fn derive_metadata(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    println!("{:?}", input);

    let cleaned_code = filter_attributes(&input.attrs, &["source"]);

    TokenStream::from(quote! {})
}
