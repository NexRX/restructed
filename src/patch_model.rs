use crate::*;
use proc_macro2::{Group, Ident, TokenStream, TokenTree};
use quote::quote;
use syn::{Attribute, DeriveInput, Type};

struct PatchModelArgs {
    name: Ident,
    omit: Vec<Ident>,
    derives: Vec<Ident>,
    default_derives: bool,
}

pub fn impl_patch_model_new(
    ast: &DeriveInput,
    attr: &Attribute,
    oai_attr: &Vec<&Attribute>,
) -> TokenStream {
    let PatchModelArgs {
        name,
        omit,
        derives,
        default_derives,
    } = parse_patch_arg(attr);

    let original_name = &ast.ident;

    // Build the fields for the new type, wrapping each original field in an Option
    let mut fields_and_is_option: Vec<(&Ident, bool)> = vec![];
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
            let option_ty = extract_type_from_option(field_ty);

            fields_and_is_option.push((field_name, option_ty.is_some()));
            fields.push(impl_struct_fields(
                field_name,
                field_ty,
                option_ty,
                &oai_f_attributes,
            ));
        }),
        _ => panic!("Patch Models can only be derived for structs"),
    };

    let derives = get_derive(default_derives, derives.iter().collect());
    let impl_from_derived = impl_from_derived(&fields_and_is_option);
    let impl_merge = impl_merge(&fields_and_is_option);
    let impl_weld_merge = impl_weld_merge(&impl_merge, original_name);

    // Generate the implementation of the PatchModel trait
    quote! {

        /// Generated patch model of [`#original_name`]
        #derives
        #(#oai_attr)*
        pub struct #name {
            #(#fields),*
        }

        impl #name  {
            /// This is what the [`From`] trait calls internally to map between the original and generated type.
            pub fn from_derived(value: #original_name) -> Self {
                Self {
                    #impl_from_derived
                }
            }

            /// Merges the updates into the given value, returning the updated value <br/>
            /// The only fields to change will be the ones that are Some. <br/>
            /// if your using the openapi feature then then only [`MaybeUndefined::Undefined`] are ingored
            pub fn merge(self, mut value: #original_name) -> #original_name {
                self.merge_mut(&mut value);
                value
            }

            /// Mutable reference version of [`Self::merge`]
            pub fn merge_mut(self, mut value: &mut #original_name) {
                #impl_merge
            }

            #impl_weld_merge

        }


        impl ::core::convert::From<#original_name> for #name  {
            fn from(value: #original_name) -> Self {
                #name::from_derived(value)
            }
        }
    }
}

fn impl_merge(fields: &[(&Ident, bool)]) -> TokenStream {
    #[cfg(feature = "openapi")]
    {
        let option_field: Vec<_> = fields
            .iter()
            .filter_map(|(ident, is_option)| is_option.then_some(ident))
            .collect();

        let required_field: Vec<_> = fields
            .iter()
            .filter_map(|(ident, is_option)| (!is_option).then_some(ident))
            .collect();

        quote! {
            #(
                match self.#required_field {
                    ::core::option::Option::Some(v) => value.#required_field = v,
                    ::core::option::Option::None => {},
                }
            )*
            #(
                match self.#option_field {
                    ::poem_openapi::types::MaybeUndefined::Value(v) => value.#option_field = ::core::option::Option::Some(v),
                    ::poem_openapi::types::MaybeUndefined::Null => value.#option_field = ::core::option::Option::None,
                    ::poem_openapi::types::MaybeUndefined::Undefined => {},
                }
            )*
        }
    }
    #[cfg(not(feature = "openapi"))]
    {
        let field: Vec<_> = fields.iter().map(|v| v.0).collect();
        quote! {
            #(
                match self.#field {
                    ::core::option::Option::Some(v) => value.#field = v,
                    ::core::option::Option::None => {},
                }
            )*
        }
    }
}

fn impl_from_derived(fields: &[(&Ident, bool)]) -> TokenStream {
    #[cfg(feature = "openapi")]
    {
        let option_field: Vec<_> = fields
            .iter()
            .filter_map(|(ident, is_option)| is_option.then_some(ident))
            .collect();

        let required_field: Vec<_> = fields
            .iter()
            .filter_map(|(ident, is_option)| (!is_option).then_some(ident))
            .collect();

        quote! {
            #(
                #option_field: ::poem_openapi::types::MaybeUndefined::from_opt_undefined(value.#option_field),
            )*
            #(
                #required_field: ::core::option::Option::Some(value.#required_field),
            )*
        }
    }
    #[cfg(not(feature = "openapi"))]
    {
        let field: Vec<_> = fields.iter().map(|v| v.0).collect();
        quote! {
            #(
                #field: ::core::option::Option::Some(value.#field),
            )*
        }
    }
}

#[allow(unused_variables)]
fn impl_weld_merge(impl_merge: &TokenStream, original_name: &Ident) -> TokenStream {
    #[cfg(feature = "weld")]
    {
        quote! {
            pub fn merge_weld(self, mut value: ::welds::state::DbState<#original_name>) -> ::welds::state::DbState<#original_name> {
                self.weld_updates_mut(&mut value);
                value
            }

            pub fn merge_weld_mut(self, mut value: &mut ::welds::state::DbState<#original_name>) {
                #impl_merge
            }
        }
    }
    #[cfg(not(feature = "weld"))]
    {
        quote!()
    }
}

fn impl_struct_fields(
    field_name: &Ident,
    field_ty: &Type,
    #[allow(unused_variables)] option_ty: Option<&Type>,
    oai_f_attr: &Vec<&Attribute>,
) -> TokenStream {
    #[cfg(feature = "openapi")]
    {
        match option_ty {
            Some(t) => {
                quote! {
                   #(#oai_f_attr)*
                   pub #field_name: ::poem_openapi::types::MaybeUndefined<#t>
                }
            }
            _ => {
                quote! {
                    #(#oai_f_attr)*
                    pub #field_name: core::option::Option<#field_ty>
                }
            }
        }
    }
    #[cfg(not(feature = "openapi"))]
    {
        quote! {
            #(#oai_f_attr)*
            pub #field_name: core::option::Option<#field_ty>
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
        return PatchModelArgs {
            name,
            omit: vec![],
            derives: vec![],
            default_derives: true,
        };
    }

    let mut args_slice = tks[2..].to_vec();

    let omit = parse_omit(&mut args_slice);
    let derives = parse_derives(&mut args_slice);
    let default_derives = parse_default_derives(&mut args_slice);
    panic_unexpected_args(vec!["fields", "derive", "derive_defaults"], &args_slice);
    PatchModelArgs {
        name,
        omit,
        derives,
        default_derives,
    }
}

fn parse_omit(args: &mut Vec<TokenTree>) -> Vec<Ident> {
    // Extract the fields args and ensuring it is a key-value pair of Ident and Group
    let fields: Group = match take_ident_group("omit", args) {
        Some(g) => g,
        None => panic!("Missing args, expected `omit(...)"),
    };

    // Parse the fields argument into a TokenStream, skip checking for commas coz lazy
    extract_idents(fields)
}

fn parse_default_derives(args: &mut Vec<TokenTree>) -> bool {
    // Extract the fields args and ensuring it is a key-value pair of Ident and Group
    take_ident_bool("default_derives", args).unwrap_or_default()
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
        let idents_of_path = path.segments.iter().fold(String::new(), |mut acc, v| {
            acc.push_str(&v.ident.to_string());
            acc.push('|');
            acc
        });
        vec!["Option|", "std|option|Option|", "core|option|Option|"]
            .into_iter()
            .find(|s| idents_of_path == *s)
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

fn has_attribute(attrs: &[syn::Attribute], name: &str, value: &str) -> bool {
    attrs.iter().any(|attr| {
        let path = attr.path();
        let segments = &path.segments;

        segments.len() == 1
            && segments[0].ident == name
            && attr
                .meta
                .require_list()
                .map_or(false, |v| v.tokens.to_string() == value)
    })
}
