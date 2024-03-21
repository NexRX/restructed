use crate::logic::{args::AttrArgs, *};
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::{self, Attribute, DataEnum, DataStruct, DeriveInput};

use self::args::ModelAttrArgs;

pub fn impl_view_model(
    ast: &DeriveInput,
    attr: &Attribute,
    defaults: ModelAttrArgs
) -> TokenStream {
    // Argument and Variable Initialization and Prep
    let (args, _) = AttrArgs::parse(attr, defaults, true);
    let AttrArgs {
        name,
        fields: _,
        derive,
        preset: _,
        attributes_with
    } = args.clone();

    let original_name = &ast.ident;
    let is_struct = matches!(&ast.data, syn::Data::Struct(_));
    let mut field_mapping: Vec<TokenStream> = vec![]; // Will contain each fields `From` trait impl
    let mut field_mapping_reverse: Vec<TokenStream> = vec![];

    // Generate Implementation
    let field_tokens: Vec<_> = match &ast.data {
        syn::Data::Struct(data) => impl_for_struct(data, &mut field_mapping, &mut field_mapping_reverse, &args),
        syn::Data::Enum(data) => impl_for_enum(data, &mut field_mapping, &mut field_mapping_reverse, &args, original_name),
        syn::Data::Union(_) => abort!(attr, "Patch Model can only be derived for `struct` & `enum`, NOT `union`"),
    };

    let structure = match is_struct {
        true => quote!(struct),
        false => quote!(enum),
    };
    

    let attributes = attributes_with.gen_top_attributes(ast);
    let derives = gen_derive(derive.as_ref());
    
    let impl_from = impl_from_trait(original_name, &name, field_mapping, field_mapping_reverse, is_struct);

    let doc_string = format!("This is a restructured (View) model of ['{original_name}']. Refer to the original model for more structual documentation.");
    quote! {
        #[doc= #doc_string]
        #derives
        #(#attributes)*
        pub #structure #name {
            #(#field_tokens),*
        }

        #impl_from
    }
}

fn impl_from_trait(
    original_name: &Ident,
    name: &Ident,
    field_mapping: Vec<TokenStream>,
    field_mapping_reverse: Vec<TokenStream>,
    is_struct: bool,
) -> proc_macro2::TokenStream {
    if is_struct {
        quote! {
            impl ::core::convert::From<#original_name> for #name  {
                fn from(value: #original_name) -> Self {
                    Self {
                        #(#field_mapping),*
                    }
                }
            }
        }
    } else {
        quote! {
            impl ::core::convert::From<#name> for #original_name  {
                fn from(value: #name) -> Self {
                    match value {
                        #(#field_mapping),*
                    }
                }
            }

            impl ::core::convert::TryFrom<#original_name> for #name {
                type Error = ();

                fn try_from(value: #original_name) -> Result<Self, Self::Error> {
                    Ok(match value {
                        #(#field_mapping_reverse, )*
                        _ => return Err(())
                    })
                }
            }
        }
    }
}


fn impl_for_struct(data: &DataStruct, field_mapping: &mut Vec<TokenStream>, field_mapping_reverse: &mut Vec<TokenStream>, args: &AttrArgs) -> Vec<TokenStream> {
    let AttrArgs {
        name: _,
        fields,
        derive: _,
        preset,
        attributes_with
    } = args;


    data
            .fields
            .iter()
            .filter(|f| {
                preset.predicate(f) && fields.predicate(f.ident.as_ref().expect("Field must be named")) 
            })
            .map(|field| {
                let vis = &field.vis;
                let docs = extract_docs(&field.attrs);
                let field_name = &field.ident.as_ref().unwrap();
                let field_ty = &field.ty;

                let field_attr = attributes_with.gen_field_attributes(field.attrs.clone());

                let mapping = quote!(#field_name: value.#field_name);
                field_mapping.push(mapping.clone());
                field_mapping_reverse.push(mapping);
                
                quote! {
                    #docs
                    #(#field_attr)*
                    #vis #field_name: #field_ty
                }
            })
            .collect()
}

fn impl_for_enum(data: &DataEnum, field_mapping: &mut Vec<TokenStream>, field_mapping_reverse: &mut Vec<TokenStream>, args: &AttrArgs, original_name: &Ident) -> Vec<TokenStream> {
    let AttrArgs {
        name,
        fields,
        derive: _,
        preset,
        attributes_with
    } = args;

    data.variants
    .iter()
    .filter(|v| fields.predicate(&v.ident))
    .map(|field| {
        let mut field_impl = field.clone();
        field_impl.attrs = attributes_with.gen_field_attributes(field_impl.attrs);

        let docs = extract_docs(&field.attrs);
        let field_name = &field.ident;

        match &field.fields {
            syn::Fields::Unit => {
                field_mapping.push(quote!{
                    #name::#field_name => #original_name::#field_name
                });
                field_mapping_reverse.push(quote!{
                    #original_name::#field_name => #name::#field_name
                });
            },
            syn::Fields::Unnamed(_) => {
                let variant_args: Vec<_> = field
                    .fields
                    .iter()
                    .filter(|f| preset.predicate(f))
                    .enumerate()
                    .map(|(i, _)| {
                        let c = (i as u8 + b'a') as char;
                        let c= syn::Ident::new(&c.to_string(), proc_macro2::Span::call_site()); // convert char to ident so it wont be "quoted"

                        quote!(#c)
                    })
                    .collect();
                field_mapping.push(quote!{
                    #name::#field_name(#(#variant_args),*) => #original_name::#field_name(#(#variant_args),*)
                });
                field_mapping_reverse.push(quote!{
                    #original_name::#field_name(#(#variant_args),*) => #name::#field_name(#(#variant_args),*)
                });
            }
            syn::Fields::Named(n) => {
                let variant_args: Vec<_> = n
                    .named
                    .iter()
                    .filter(|f| preset.predicate(f))
                    .map(|f| {

                        let arg = f.ident.as_ref();
                        quote!(#arg)
                    })
                    .collect();
                field_mapping.push(quote!{
                    #name::#field_name{#(#variant_args),*} => #original_name::#field_name{#(#variant_args),*}
                });
                field_mapping_reverse.push(quote!{
                    #original_name::#field_name(#(#variant_args),*) => #name::#field_name(#(#variant_args),*)
                });
            },
        };
        
        quote! {
            #docs
            #field_impl
        }
    })
    .collect()

}