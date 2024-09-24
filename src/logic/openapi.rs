use syn::Attribute;
use quote::quote;
use proc_macro2::{Ident, TokenStream};
use super::args::ModelAttrArgs;

/// Checks if the model should derive the `Example` utilising the parents impl
pub(crate) fn impl_oai_example(name: &Ident, original_name: &Ident, model_args: &ModelAttrArgs) -> TokenStream {
    match model_args.extras.has_oai_example {
       true => quote!{
           impl ::poem_openapi::types::Example for #name {
               fn example() -> Self {
                   #original_name::example().into()
               }
           }
       },
       false => quote!(),
    }
}

pub(crate) fn has_oai_attribute(attrs: &[Attribute], containing: Option<&str>) -> bool {
    attrs.iter()
        .filter(|a| a.path().is_ident("oai"))
        .filter(|a| 
            match containing {
                Some(name) =>
                    a.meta.require_list()
                    .expect("oai attribute usually has a list")
                    .tokens
                    .to_string()
                    .contains(name),
                None => true
            }
        )
        .count() > 0
}
