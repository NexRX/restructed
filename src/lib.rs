#![doc = include_str!("../readme.md")]

mod logic;
mod patch;
mod view;

use crate::logic::is_attribute;
use logic::args::{ExtraConditions, ModelAttrArgs};
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro_derive(Models, attributes(model, view, patch))]
pub fn models(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let model_attr = ast.attrs
        .iter()
        .filter(|v| is_attribute(v, "model"))
        .cloned()
        .collect::<Vec<_>>();

    let extras = ExtraConditions::parse(&ast.attrs);
    let model_args = ModelAttrArgs::parse(model_attr, extras);

    let views: Vec<proc_macro2::TokenStream> = ast
        .attrs
        .iter()
        .filter(|v| is_attribute(v, "view"))
        .map(|a| view::impl_view_model(&ast, a, &model_args))
        .collect();

    let patches: Vec<proc_macro2::TokenStream> = ast
        .attrs
        .iter()
        .filter(|v| is_attribute(v, "patch"))
        .map(|a| patch::impl_patch_model(&ast, a, &model_args))
        .collect();

    let gen = quote::quote!(
        #(#views)*
        #(#patches)*
    );

    gen.into()
}
