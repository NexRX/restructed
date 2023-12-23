use crate::*;
use proc_macro2::{Ident, TokenStream, TokenTree};
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
) -> TokenStream {
    let ViewModelArgs {
        name,
        fields,
        derives,
        default_derives,
    } = parse_view_attributes(attr);

    // // Filter attr to only copy the 'oai' ones over
    let original_name = &ast.ident;
    let mut is_struct = true;
    let mut field_from_mapping: Vec<TokenStream> = vec![];

    let field_tokens: Vec<_> = match &ast.data {
        syn::Data::Struct(data) => data
            .fields
            .iter()
            .filter(|f| fields.contains(f.ident.as_ref().expect("Field(s) must be named")))
            .map(|field| {
                let field_attr: &Vec<syn::Attribute> = &field.attrs;

                let vis = &field.vis;
                let field_name = &field.ident.as_ref().unwrap();
                let field_ty = &field.ty;
                let oai_f_attributes: Vec<_> = extract_oai_f_attributes(field_attr);

                field_from_mapping.push(quote!(#field_name: value.#field_name));
                quote! {
                   #(#oai_f_attributes)*
                   #vis #field_name: #field_ty
                }
            })
            .collect(),
        syn::Data::Enum(data) => {
            is_struct = false;
            data.variants
                .iter()
                .filter(|v| fields.contains(&v.ident))
                .map(|field| {
                    let oai_f_attr = get_oai_attributes(&field.attrs);

                    let mut field_without_attrs = field.clone();
                    field_without_attrs.attrs = vec![];

                    let field_name = &field.ident;

                    match &field.fields {
                        syn::Fields::Unit => {
                            field_from_mapping.push(quote!{
                                #name::#field_name => #original_name::#field_name
                            });
                        },
                        syn::Fields::Unnamed(_) => {
                            let variant_args: Vec<_> = field
                                .fields
                                .iter()
                                .enumerate()
                                .map(|(i, _)| {
                                    let c = (i as u8 + b'a') as char;
                                    // convert c to ident so it wont be quoted
                                    let c= syn::Ident::new(&c.to_string(), proc_macro2::Span::call_site());

                                    quote!(#c)
                                })
                                .collect();
                            field_from_mapping.push(quote!{
                                #name::#field_name(#(#variant_args),*) => #original_name::#field_name(#(#variant_args),*)
                            });
                        }
                        syn::Fields::Named(n) => {
                            let variant_args: Vec<_> = n
                                .named
                                .iter()
                                .map(|f| {
                                    let arg = f.ident.as_ref();
                                    quote!(#arg)
                                })
                                .collect();
                            field_from_mapping.push(quote!{
                                #name::#field_name{#(#variant_args),*} => #original_name::#field_name{#(#variant_args),*}
                            });
                        },
                    };
                    
                    quote! {
                        #(#oai_f_attr)*
                        #field_without_attrs
                    }
                })
                .collect()
        }
        _ => abort!(attr, "Patch Model can only be derived for structs & enums"),
    };

    let data_type = match is_struct {
        true => quote!(struct),
        false => quote!(enum),
    };
    let derives = get_derive(default_derives, derives.iter().collect(), is_struct);
    let impl_from = impl_from_trait(original_name, &name, field_from_mapping, is_struct);

    quote! {
        /// A generated view of the original struct with only the specified fields
        #derives
        #(#oai_attr)*
        pub #data_type #name {
            #(#field_tokens),*
        }

        #impl_from
    }
}

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
        x => {
            abort!(
                x,
                "First argument must be an identifier (name) of the struct for the view"
            )
        }
    };
    if tks.len() < 3 {
        abort!(attr, "Invalid syntax, expected at least one argument");
    }
    let mut args_slice = tks[2..].to_vec();

    let fields = parse_fields(&mut args_slice, attr);
    let derives = parse_derives(&mut args_slice);
    let default_derives = parse_default_derives(&mut args_slice);
    abort_unexpected_args(vec!["fields", "derive", "default_derives"], &args_slice);

    ViewModelArgs {
        name,
        fields,
        derives,
        default_derives,
    }
}

/// Parse a list of identifiers equal to fields we want in the model. Aborts if none are found.
fn parse_fields(args: &mut Vec<TokenTree>, attr_spanned: &Attribute) -> Vec<Ident> {
    // Extract the fields args and ensuring it is a key-value pair of Ident and Group
    let fields: Group = match take_ident_group("fields", args) {
        Some(g) => g,
        None => abort!(attr_spanned, "Missing args, expected `fields(...)"),
    };

    // Parse the fields argument into a TokenStream, skip checking for commas coz lazy
    extract_idents(fields)
}
