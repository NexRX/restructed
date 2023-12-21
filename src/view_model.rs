use core::panic;

use crate::*;
use proc_macro2::{Ident, TokenTree};
use quote::quote;
use syn::{self, Attribute, DeriveInput};

struct ViewModelArgs {
    name: Ident,
    fields: Vec<Ident>,
    derives: Vec<Ident>,
    default_derives: bool,
}

pub fn impl_view_model(
    ast: &DeriveInput,
    attr: &Attribute,
    oai_attr: &Vec<&Attribute>,
) -> proc_macro2::TokenStream {
    let ViewModelArgs {
        name,
        fields,
        derives,
        default_derives,
    } = parse_view_attributes(attr);

    // // Filter attr to only copy the 'oai' ones over
    let original_name = &ast.ident;

    let mut field_names: Vec<_> = vec![];
    let field_tokens: Vec<_> = match &ast.data {
        syn::Data::Struct(data) => data
            .fields
            .iter()
            .filter(|f| fields.contains(f.ident.as_ref().expect("Field(s) must be named")))
            .map(|field| {
                let field_attributes: &Vec<syn::Attribute> = &field.attrs;

                let field_name = &field.ident.as_ref().unwrap();
                let field_ty = &field.ty;
                let oai_f_attributes: Vec<_> = field_attributes
                    .iter()
                    .filter(|attr| {
                        attr.meta
                            .path()
                            .segments
                            .first()
                            .map_or(false, |seg| seg.ident == "oai")
                    })
                    .collect();

                field_names.push(field_name.to_owned());
                quote! {
                   #(#oai_f_attributes)*
                   pub #field_name: #field_ty
                }
            })
            .collect(),
        _ => panic!("PatchModel can only be derived for structs"),
    };

    let derives = get_derive(default_derives, derives.iter().collect());

    quote! {
        /// A generated view of the original struct with only the specified fields
        #derives
        #(#oai_attr)*
        pub struct #name {
            #(#field_tokens),*
        }

        impl ::core::convert::From<#original_name> for #name  {
            fn from(value: #original_name) -> Self {
                Self {
                    #(#field_names: value.#field_names),*
                }
            }
        }
    }
}

fn parse_view_attributes(attr: &Attribute) -> ViewModelArgs {
    let tks: Vec<TokenTree> = attr
        .meta
        .require_list()
        .unwrap()
        .to_owned()
        .tokens
        .into_iter()
        .collect::<Vec<_>>();

    let name = match &tks[0] {
        TokenTree::Ident(v) => v.clone(),
        _ => {
            panic!("First argument must be an identifier (name) of the struct for the view")
        }
    };
    if tks.len() < 3 {
        panic!("Invalid syntax, expected at least one argument");
    }
    let mut args_slice = tks[2..].to_vec();

    let fields = parse_fields(&mut args_slice);
    let derives = parse_derives(&mut args_slice);
    let default_derives = parse_default_derives(&mut args_slice);
    panic_unexpected_args(vec!["fields", "derive", "default_derives"], &args_slice);

    ViewModelArgs {
        name,
        fields,
        derives,
        default_derives,
    }
}

/// Parse a list of identifiers equal to fields we want in the model. Panics if none are found.
fn parse_fields(args: &mut Vec<TokenTree>) -> Vec<Ident> {
    // Extract the fields args and ensuring it is a key-value pair of Ident and Group
    let fields: Group = match take_ident_group("fields", args) {
        Some(g) => g,
        None => panic!("Missing args, expected `fields(...)"),
    };

    // Parse the fields argument into a TokenStream, skip checking for commas coz lazy
    extract_idents(fields)
}
