use crate::logic::{
    args::{AttrArgs, OptionType},
    *,
};
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::{Attribute, DeriveInput, Type};

pub fn impl_patch_model(ast: &DeriveInput, attr: &Attribute) -> TokenStream {
    // Argument and Variable Initialization and Prep
    let (args, mut remainder) = AttrArgs::parse(attr, false);
    let AttrArgs {
        name,
        fields: _,
        derive,
        preset,
        attributes_with,
    } = args.clone();

    let option = OptionType::parse(&mut remainder).unwrap_or_else(|| preset.option());

    AttrArgs::abort_unexpected(&remainder, &["option"]);

    let original_name = &ast.ident;

    // Build the fields for the new type, wrapping each original field in an Option
    let mut fields_and_is_option: Vec<(&Ident, bool)> = vec![];
    let mut fields: Vec<_> = vec![];
    match &ast.data {
        syn::Data::Struct(data) => data
            .fields
            .iter()
            .filter(|f| {
                preset.predicate(f)
                    && args
                        .fields
                        .predicate(f.ident.as_ref().expect("Field must be named"))
            })
            .for_each(|field| {
                let field_name = &field.ident.as_ref().unwrap();

                // Add
                let docs = extract_docs(&field.attrs);
                let field_ty = &field.ty;
                let option_ty = extract_type_from_option(field_ty);

                fields_and_is_option.push((field_name, option_ty.is_some()));
                fields.push(impl_struct_fields(
                    field_name, field_ty, option_ty, &docs, option,
                ));
            }),
        _ => abort!(attr, "Patch Models can only be derived for structs"),
    };

    let attributes = attributes_with.gen_top_attributes(ast);
    let derives = gen_derive(derive.as_ref());
    let impl_from_derived = impl_from_derived(&fields_and_is_option, option);
    let impl_merge = impl_merge(&fields_and_is_option, option);

    // Generate the implementation of the PatchModel trait
    quote! {

        /// Generated patch model of [`#original_name`]
        #derives
        #(#attributes)*
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
        }


        impl ::core::convert::From<#original_name> for #name  {
            fn from(value: #original_name) -> Self {
                #name::from_derived(value)
            }
        }
    }
}

fn impl_merge(fields: &[(&Ident, bool)], option: OptionType) -> TokenStream {
    match option {
        OptionType::MaybeUndefined => {
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
        OptionType::Option => {
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
}

fn impl_from_derived(fields: &[(&Ident, bool)], option: OptionType) -> TokenStream {
    match option {
        OptionType::MaybeUndefined => {
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
        OptionType::Option => {
            let field: Vec<_> = fields.iter().map(|v| v.0).collect();
            quote! {
                #(
                    #field: ::core::option::Option::Some(value.#field),
                )*
            }
        }
    }
}

fn impl_struct_fields(
    field_name: &Ident,
    field_ty: &Type,
    #[allow(unused_variables)] option_ty: Option<&Type>,
    docs: &TokenStream,
    option: OptionType,
) -> TokenStream {
    match option {
        OptionType::MaybeUndefined => match option_ty {
            Some(t) => {
                quote! {
                    #docs
                    pub #field_name: ::poem_openapi::types::MaybeUndefined<#t>
                }
            }
            _ => {
                quote! {
                    #docs
                    pub #field_name: core::option::Option<#field_ty>
                }
            }
        },
        OptionType::Option => {
            quote! {
                #docs
                pub #field_name: core::option::Option<#field_ty>
            }
        }
    }
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
