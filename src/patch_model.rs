use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenTree};
use quote::{format_ident, quote};
use syn::{self, parse_macro_input, Attribute, DeriveInput, Type};

use crate::*;

struct PatchModelArgs {
    name: Ident,
    omit: Vec<Ident>,
}

pub fn impl_patch_model_new(
    ast: &DeriveInput,
    attr: &Attribute,
    oai_attr: &Vec<&Attribute>,
) -> proc_macro2::TokenStream {
    let PatchModelArgs { name, omit } = parse_patch_arg(attr);

    let original_name = &ast.ident;

    // Build the fields for the new type, wrapping each original field in an Option
    let mut option_field: Vec<Ident> = vec![];
    let mut maybe_field: Vec<Ident> = vec![];
    let mut fields: Vec<_> = vec![];
    match &ast.data {
        syn::Data::Struct(data) => data.fields.iter().for_each(|field| {
            let field_attributes: &Vec<syn::Attribute> = &field.attrs;
            let field_name = &field.ident.as_ref().unwrap();

            // Omit
            if omit.contains(field_name) || has_attribute(field_attributes, "oai", "read_only") {
                return;
            }

            // Add
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

            let field = match field_ty {
                Type::Path(_) if extract_type_from_option(field_ty).is_some() => {
                    let t = extract_type_from_option(field_ty).unwrap();
                    maybe_field.push(field.ident.to_owned().unwrap());
                    quote! {
                       #(#oai_f_attributes)*
                       pub #field_name: ::poem_openapi::types::MaybeUndefined<#t>
                    }
                }
                _ => {
                    option_field.push(field.ident.to_owned().unwrap());
                    quote! {
                        #(#oai_f_attributes)*
                        pub #field_name: core::option::Option<#field_ty>
                    }
                }
            };
            fields.push(field);
        }),
        _ => panic!("Patch Models can only be derived for structs"),
    };

    // Generate the implementation of the PatchModel trait
    quote! {

        // Define the new type
        #[derive(Debug, Default, ::poem_openapi::Object, Clone, PartialEq, Eq, PartialOrd, Ord)]
        #(#oai_attr)*
        pub struct #name {
            #(#fields),*
        }

        impl #name  {
            pub fn from_derived(value: #original_name) -> Self {
                Self {
                    #(
                        #maybe_field: ::poem_openapi::types::MaybeUndefined::from_opt_undefined(value.#maybe_field),
                    )*
                    #(
                        #option_field: ::core::option::Option::Some(value.#option_field),
                    )*
                }
            }

            pub fn merge_updates(self, mut value: #original_name) -> #original_name {
                self.merge_updates_mut(&mut value);
                value
            }

            pub fn merge_updates_mut(self, mut value: &mut #original_name) {
                #(
                    match self.#option_field {
                        ::core::option::Option::Some(v) => value.#option_field = v,
                        ::core::option::Option::None => {},
                    }
                )*
                #(
                    match self.#maybe_field {
                        ::poem_openapi::types::MaybeUndefined::Value(v) => value.#maybe_field = ::core::option::Option::Some(v),
                        ::poem_openapi::types::MaybeUndefined::Null => value.#maybe_field = ::core::option::Option::None,
                        ::poem_openapi::types::MaybeUndefined::Undefined => {},
                    }
                )*
            }

            pub fn weld_updates(self, mut value: ::welds::state::DbState<#original_name>) -> ::welds::state::DbState<#original_name> {
                self.weld_updates_mut(&mut value);
                value
            }

            pub fn weld_updates_mut(self, mut value: &mut ::welds::state::DbState<#original_name>) {
                #(
                    match self.#option_field {
                        ::core::option::Option::Some(v) => value.#option_field = v,
                        ::core::option::Option::None => {},
                    }
                )*
                #(
                    match self.#maybe_field {
                        ::poem_openapi::types::MaybeUndefined::Value(v) => value.#maybe_field = ::core::option::Option::Some(v),
                        ::poem_openapi::types::MaybeUndefined::Null => value.#maybe_field = ::core::option::Option::None,
                        ::poem_openapi::types::MaybeUndefined::Undefined => {},
                    }
                )*
            }
        }


        impl ::core::convert::From<#original_name> for #name  {
            fn from(value: #original_name) -> Self {
                #name::from_derived(value)
            }
        }
    }
}

fn parse_patch_arg(attr: &Attribute) -> PatchModelArgs {
    let tks = attr
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
        return PatchModelArgs { name, omit: vec![] };
    }

    let args_slice = &tks[2..];
    panic_unexpected_args(vec!["omit"], args_slice);

    let omit = parse_omit(args_slice);

    PatchModelArgs { name, omit }
}

fn parse_omit(args: &[TokenTree]) -> Vec<Ident> {
    // Extract the fields args and ensuring it is a key-value pair of Ident and Group
    let fields: &Group = match get_ident_group("omit", args) {
        Some(g) => g,
        None => panic!("Missing args, expected `omit(...)"),
    };

    // Parse the fields argument into a TokenStream, skip checking for commas coz lazy
    extract_idents(fields)
}

fn extract_type_from_option(ty: &Type) -> Option<&Type> {
    use syn::{GenericArgument, Path, PathArguments, PathSegment};

    fn extract_type_path(ty: &syn::Type) -> Option<&Path> {
        match *ty {
            syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
            _ => None,
        }
    }

    fn extract_option_segment(path: &Path) -> Option<&PathSegment> {
        let idents_of_path = path
            .segments
            .iter()
            .into_iter()
            .fold(String::new(), |mut acc, v| {
                acc.push_str(&v.ident.to_string());
                acc.push('|');
                acc
            });
        vec!["Option|", "std|option|Option|", "core|option|Option|"]
            .into_iter()
            .find(|s| &idents_of_path == *s)
            .and_then(|_| path.segments.last())
    }

    extract_type_path(ty)
        .and_then(|path| extract_option_segment(path))
        .and_then(|path_seg| {
            let type_params = &path_seg.arguments;
            // It should have only on angle-bracketed param ("<String>"):
            match *type_params {
                PathArguments::AngleBracketed(ref params) => params.args.first(),
                _ => None,
            }
        })
        .and_then(|generic_arg| match *generic_arg {
            GenericArgument::Type(ref ty) => Some(ty),
            _ => None,
        })
}

fn has_attribute(attrs: &Vec<syn::Attribute>, name: &str, value: &str) -> bool {
    attrs.iter().any(|attr| {
        let path = attr.path();
        let segments = &path.segments;

        segments.len() == 1
            && segments[0].ident.to_string() == name
            && attr
                .meta
                .require_list()
                .map_or(false, |v| v.tokens.to_string() == value)
    })
}
