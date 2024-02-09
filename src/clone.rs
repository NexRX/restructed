use crate::*;
use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::quote;
use syn::{self, Attribute, DeriveInput};

struct CloneModelArgs {
    name: Ident,
    derives: Vec<Ident>,
}

pub fn impl_clone_model(ast: &DeriveInput, attr: &Attribute) -> TokenStream {
    let CloneModelArgs { name, derives } = parse_clone_attributes(attr);

    let attrs: Vec<_> = ast
        .attrs
        .iter()
        .filter(|v| is_attribute(v, "view")) 
        .filter(|v| is_attribute(v, "patch")) 
        .filter(|v| is_attribute(v, "clone")) 
        .collect();
    let vis = &ast.vis;
    #[allow(unused_assignments)]
    let mut data_type = TokenStream::default();
    let data = match &ast.data {
        syn::Data::Struct(v) => {
            data_type = quote!(struct);
            let fields = &v.fields;
            quote!(#fields)
        }
        syn::Data::Enum(v) => {
            data_type = quote!(enum);
            let fields = &v.variants;
            quote!(#fields)
        }
        syn::Data::Union(v) => {
            data_type = quote!(v.union_token.clone());
            let fields = &v.fields;
            quote!(#fields)
        }
    };

    quote! {
        #[derive(#(#derives),*)]
        #( #attrs )*
        #vis #data_type #name
            #data
    }
}

// TODO: Impl this for struct, union, and enum
fn impl_from_trait(
    original_name: &Ident,
    name: &Ident,
    field_from_mapping: Vec<TokenStream>,
    is_struct: bool,
) -> proc_macro2::TokenStream {
    if is_struct {
        quote! {
            impl ::core::convert::From<#original_name> for #name  {
                fn from(value: #original_name) -> Self {
                    Self {
                        #(#field_from_mapping),*
                    }
                }
            }
        }
    } else {
        quote! {
            impl ::core::convert::From<#name> for #original_name  {
                fn from(value: #name) -> Self {
                    match value {
                        #(#field_from_mapping),*
                    }
                }
            }
        }
    }
}

fn parse_clone_attributes(attr: &Attribute) -> CloneModelArgs {
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
        x => {
            abort!(
                x,
                "First argument must be an identifier (name) of the struct for the clone"
            )
        }
    };

    let mut args_slice = tks[2..].to_vec();
    let derives = parse_derives(&mut args_slice);
    abort_unexpected_args(vec!["derive"], &args_slice);

    CloneModelArgs { name, derives }
}
