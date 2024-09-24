use args::ModelAttrArgs;
use proc_macro2::{Group, Ident, Literal, TokenStream, TokenTree};
use proc_macro_error::abort;
use quote::quote;
use syn::{parse2, Attribute};

#[cfg(test)]
mod tests;
#[cfg(feature = "openapi")]
mod openapi;
pub(crate) use openapi::*;

pub(crate) mod args;

/// Check the first segment of an attribute to see if it matches the given name
pub(crate) fn is_attribute(attr: &Attribute, name: &str) -> bool {
    get_attribute_name(attr).map_or(false, |v| v == name)
}

/// Gets the first segment of the attribute which should typically the name of the attribute
pub(crate) fn get_attribute_name(attr: &Attribute) -> Option<&Ident> {
    attr.meta.path().segments.first().map(|v| &v.ident)
}

//  "Invalid or missing `{name}` argument, expected a group of args, e.g. `{name}(...)`"

/// Extract a group for a given identifier, e.g. `name(...)`. The `(...)` part is returned.
pub(crate) fn take_ident_group(name: &str, args: &mut Vec<TokenTree>) -> Option<Group> {
    for (i, tk) in args.iter().enumerate() {
        match tk {
            TokenTree::Ident(v) if *v == name => match args.get(i + 1) {
                Some(TokenTree::Group(g)) => {
                    let g = g.to_owned();
                    args.remove(i); // Remove Ident
                    args.remove(i); // Remove Group
                    if matches!(args.get(i), Some(TokenTree::Punct(v)) if v.as_char() == ',') {
                        args.remove(i); // Remove leading Comma
                    }
                    return Some(g);
                },
                e => abort!(e, "Invalid or missing `{name}` argument, expected a group of args, e.g. `{name}(...)`"),
            },
            _ => {}
        }
    }
    None
}

/// Extract a literal for a given identifier, e.g. `name = "..."`. The `"..."` part is returned.
#[allow(dead_code)]
pub(crate) fn take_ident_bool(name: &str, args: &mut Vec<TokenTree>) -> Option<bool> {
    for (i, tk) in args.iter().enumerate() {
        match tk {
            TokenTree::Ident(v) if *v == name => match args.get(i + 1) {
                Some(TokenTree::Punct(p)) if p.as_char() == '=' => {
                    let value = args
                        .get(i + 2)
                        .map(|v| v.to_string())
                        .unwrap_or("Nothing".to_string());
                    let b =  match value == "true" || value == "false" {
                        true => value == "true",
                        false => abort!(
                           tk, "Invalid or missing `{name}` argument, expected a bool, e.g. `{name} = true`"
                        )
                    };

                    args.remove(i); // Remove Ident
                    args.remove(i); // Remove Punct
                    args.remove(i); // Remove Literal (Bool)
                    if matches!(args.get(i), Some(TokenTree::Punct(v)) if v.as_char() == ',') {
                        args.remove(i); // Remove leading Comma
                    }
                    return Some(b);
                }
                _ => abort!(
                    tk,
                    "Invalid or missing `{name}` argument, expected a bool, e.g. `{name} = true`"
                ),
            },
            _ => {}
        }
    }
    None
}

pub(crate) fn take_ident_ident(name: &str, args: &mut Vec<TokenTree>) -> Option<Ident> {
    for (i, tk) in args.iter().enumerate() {
        match tk {
            TokenTree::Ident(v) if *v == name => match args.get(i + 1) {
                Some(TokenTree::Punct(p)) if p.as_char() == '=' => {
                    let value = match args
                        .get(i + 2) {
                            Some(proc_macro2::TokenTree::Ident(v))  => v.to_owned(),
                            Some(v) => abort!(v, "Invalid arguement value, expected a identifier, e.g. `{} = *StructIdentifer*` but got {}", name, v),
                            None => abort!(v, "Missing `{name}` argument, expected an identifier, e.g. `{} = MyStruct`", name),
                        };
                    
                    args.remove(i); // Remove Ident {name}
                    args.remove(i); // Remove Punct =
                    args.remove(i); // Remove Ident {value}
                    if matches!(args.get(i), Some(TokenTree::Punct(v)) if v.as_char() == ',') {
                        args.remove(i); // Remove leading Comma
                    }
                    return Some(value);
                }
                _ => abort!(
                    tk,
                    "Invalid or missing `{name}` argument, expected an identifier, e.g. `{name} = MyStruct`"
                ),
            },
            _ => {}
        }
    }
    None
}

pub(crate) fn take_ident_literal(name: &str, args: &mut Vec<TokenTree>) -> Option<Literal> {
    for (i, tk) in args.iter().enumerate() {
        match tk {
            TokenTree::Ident(v) if *v == name => match args.get(i + 1) {
                Some(TokenTree::Punct(p)) if p.as_char() == '=' => {
                    let value = match args
                        .get(i + 2) {
                            Some(proc_macro2::TokenTree::Literal(v))  => v.to_owned(),
                            Some(v) => abort!(v, "Invalid arguement value, expected a identifier, e.g. `{} = *StructIdentifer*` but got {}", name, v),
                            None => abort!(v, "Missing `{name}` argument, expected an identifier, e.g. `{} = MyStruct`", name),
                        };
                    
                    args.remove(i); // Remove Ident {name}
                    args.remove(i); // Remove Punct =
                    args.remove(i); // Remove Literal {value}
                    if matches!(args.get(i), Some(TokenTree::Punct(v)) if v.as_char() == ',') {
                        args.remove(i); // Remove leading Comma
                    }
                    return Some(value);
                }
                _ => abort!(
                    tk,
                    "Invalid or missing `{name}` argument, expected an identifier, e.g. `{name} = MyStruct`"
                ),
            },
            _ => {}
        }
    }
    None
}

/// Extract a group for a given identifier, e.g. `name(...)`. The `(...)` part is returned. (Returns a group of syn::Path)
/// This function creates a stream/iter does 3 main things
/// 1. Filter for indices of all commas
/// 2. Create a range between indices (this will be the range of all paths)
/// 3. Parse the ranges into `syn::Path`
pub(crate) fn take_path_group(name: &str, args: &mut Vec<TokenTree>) -> Option<Vec<syn::Path>> {
    let g = take_ident_group(name, args)?;
    let paths: Vec<syn::Path> = g
        .stream()
        .into_iter()
        .enumerate() // Step 1 (Collect all comma indices)
        .filter_map(|(i, v)| match v {
            TokenTree::Punct(p) if p.as_char() == ',' => Some(i),
            _ => None,
        }) // Step 2 (Collect all ranges)
        .chain(std::iter::once(g.stream().into_iter().count()))
        .scan(0, |state, next| {
            let start = match state {
                0 => 0,
                _ => *state + 1,
            };

            let range = start..next;
            *state = next;

            match start == next {
                true => None,
                false => Some(range),
            }
        })
        .map(|range| {
            // Step 3 (Parse all paths)
            let path: proc_macro2::TokenStream = g
                .stream()
                .into_iter()
                .skip(range.start)
                .take(range.end - range.start)
                .collect();
            match parse2(path) {
                Ok(v) => v,
                Err(_) => abort!(g, "Invalid, not a valid derive Path (e.g. `core::fmt::Debug`) or Identifier (e.g. `Debug`)")
            }
        })
        .collect();
    match paths.is_empty() {
        true => None,
        false => Some(paths),
    }
}

/// Parse the fields argument into a TokenStream, skipping checking for commas coz lazy
pub(crate) fn extract_idents(group: Group) -> Vec<Ident> {
    group
        .stream()
        .into_iter()
        .filter_map(|tt| match tt {
            TokenTree::Ident(ident) => Some(ident),
            TokenTree::Punct(v) if v.as_char() == ',' => None,
            tt => abort!(tt, "Invalid syntax, expected a field identifier, got {tt}`"),
        })
        .collect()
}


pub(crate) fn is_doc(v: &&Attribute) -> bool {
    v.meta.require_name_value().map_or(false, |v| {
        v.path.segments.first().map_or(false, |v| v.ident == "doc")
    })
}

pub(crate) fn extract_docs(attrs: &[Attribute]) -> proc_macro2::TokenStream {
    let docs: Vec<_> = attrs.iter().filter(is_doc).collect();
    quote!(#(#docs)*)
}

/// Aborts on unexpected args to show that they arent valid
pub(crate) fn abort_unexpected_args(names: Vec<&str>, args: &[TokenTree]) {
    for tk in args.iter() {
        match tk {
            TokenTree::Ident(v) if names.contains(&v.to_string().as_str()) => {}
            TokenTree::Ident(v) => {
                abort!(
                    v,
                    "Unknown argument `{}`, all known arguments are {:?}",
                    v,
                    names
                )
            }
            _ => {}
        }
    }
}

pub(crate) fn gen_derive(
    from_args: Option<&Vec<syn::Path>>
) -> proc_macro2::TokenStream {
    let mut derives: Vec<proc_macro2::TokenStream> = vec![];

    match from_args {
        Some(from_args) if !from_args.is_empty() => derives.push(quote!(#(#from_args),*)),
        _ => {}
    }

    quote!(
        #[derive(#(#derives),*)]
    )
}

/// Generates extra nice to have implementations for the generated models
 // Dev Note: Okay for now in generic logic module, but future extras may need to be handled in patch/view modules.
 pub fn impl_extras(
    original_name: &Ident,
    name: &Ident,
    model_args: &ModelAttrArgs,
) -> Vec<TokenStream> {
    vec![
        #[cfg(feature = "openapi")]
        impl_oai_example(name, original_name, model_args)
    ]
}