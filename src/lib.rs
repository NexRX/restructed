#![doc = include_str!("../readme.md")]

mod patch;
mod view;

use proc_macro::TokenStream;
use proc_macro2::{Group, Ident, TokenTree};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::Attribute;

/// Derives any number of models that are a subset of the struct deriving this macro. There are two types of models possible. <br/>
/// # view
///A selective subset of fields from the original model of the same types.
///
///**Arguements:**
///- `name` - The name of the struct the generate (**Required**, **Must be first** e.g. `MyStruct`)
///- `fields` - A *list* of field names in the original structure to carry over (**Required**, e.g. `fields(field1, field2, ...)`)
///- `derive` - A *list* of derivables (in scope) to derive on the generated struct (e.g. `derive(Clone, Debug, thiserror::Error)`)
///- `default_derives` - A *bool*, if `true` *(default)* then the a list of derives will be additionally derived. Otherwise, `false` to avoid this (e.g. `default_derives = false`)
///
///**Example:**
///```rust
///   // Original
///   #[derive(restructed::Models)]
///   #[view(UserProfile, fields(display_name, bio), derive(Clone), default_derives = false)]
///   struct User {
///       id: i32,
///       display_name: String,
///       bio: String,
///       password: String,
///   }
///```
///Generates:
///```rust
///   #[derive(Clone)]
///   struct UserProfile {
///       display_name: String,
///       bio: String,
///   }
///```
///
///# patch
///A complete subset of fields of the original model wrapped in `Option<T>` with the ability to omit instead select fields.
///
///**Arguements:**
///- `name` - The name of the struct the generate (**Required**, **Must be first** e.g. `MyStruct`)
///- `omit` - A *list* of field names in the original structure to omit (**Required**, e.g. `fields(field1, field2, ...)`)
///- `derive` - A *list* of derivables (in scope) to derive on the generated struct (e.g. `derive(Clone, Debug, thiserror::Error)`)
///- `default_derives` - A *bool*, if `true` *(default)* then the a list of derives will be additionally derived. Otherwise, `false` to avoid this (e.g. `default_derives = false`)
///
///**Example:**
///```rust
///   // Original
///   #[derive(restructed::Models)]
///   #[patch(UserUpdate, omit(id))]
///   struct User {
///      id: i32,
///      display_name: String,
///      bio: String,
///      password: String,
///   }
///```
///
///Generates:
///```rust
///   #[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)] // <-- Default derives (when *not* disabled)
///   struct UserUpdate {
///       display_name: Option<String>,
///       bio: Option<String>, // MaybeUndefined<String> with feature 'openapi'
///       password: Option<String>,
///   }
///```
///
/// For more information, read the crate level documentation.
#[proc_macro_error]
#[proc_macro_derive(Models, attributes(view, patch))]
pub fn models(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let oai_attr = get_oai_attributes(&ast.attrs);

    let views: Vec<proc_macro2::TokenStream> = ast
        .attrs
        .iter()
        .filter(|v| is_attribute(v, "view"))
        .map(|a| view::impl_view_model(&ast, a, &oai_attr))
        .collect();

    let patches: Vec<proc_macro2::TokenStream> = ast
        .attrs
        .iter()
        .filter(|v| is_attribute(v, "patch"))
        .map(|a| patch::impl_patch_model(&ast, a, &oai_attr))
        .collect();

    let gen = quote::quote!(
        #(#views)*
        #(#patches)*
    );

    gen.into()
}

// TODO: this stuff slaps. i should make it public

// Supporting Functions

fn get_oai_attributes(attrs: &[Attribute]) -> Vec<&Attribute> {
    attrs
        .iter()
        .filter(|attr| is_attribute(attr, "oai"))
        .collect()
}

/// Check the first segment of an attribute to see if it matches the given name
fn is_attribute(attr: &Attribute, name: &str) -> bool {
    get_attribute_name(attr).map_or(false, |v| v == name)
}

/// Gets the first segment of the attribute which should typically the name of the attribute
fn get_attribute_name(attr: &Attribute) -> Option<&Ident> {
    attr.meta.path().segments.first().map(|v| &v.ident)
}

//  "Invalid or missing `{name}` argument, expected a group of args, e.g. `{name}(...)`"

/// Extract a group for a given identifier, e.g. `name(...)`. The `(...)` part is returned.
fn take_ident_group(name: &str, args: &mut Vec<TokenTree>) -> Option<Group> {
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
fn take_ident_bool(name: &str, args: &mut Vec<TokenTree>) -> Option<bool> {
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

/// Parse the fields argument into a TokenStream, skipping checking for commas coz lazy
fn extract_idents(group: Group) -> Vec<Ident> {
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

fn extract_oai_f_attributes(attrs: &[syn::Attribute]) -> Vec<&syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| {
            attr.meta
                .path()
                .segments
                .first()
                .map_or(false, |seg| seg.ident == "oai")
        })
        .collect()
}

fn is_doc(v: &&Attribute) -> bool {
    v.meta.require_name_value().map_or(false, |v| {
        v.path.segments.first().map_or(false, |v| v.ident == "doc")
    })
}

fn extract_docs(attrs: &[Attribute]) -> proc_macro2::TokenStream {
    let docs: Vec<_> = attrs.iter().filter(is_doc).collect();
    quote!(#(#docs)*)
}

/// Aborts on unexpected args to show that they arent valid
fn abort_unexpected_args(names: Vec<&str>, args: &[TokenTree]) {
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
/// Parse a list of identifiers we want to derive. Will be empty if none are found.
fn parse_derives(args: &mut Vec<TokenTree>) -> Vec<Ident> {
    // Extract the fields args and ensuring it is a key-value pair of Ident and Group
    let fields: Group = match take_ident_group("derive", args) {
        Some(g) => g,
        None => return vec![],
    };

    extract_idents(fields)
}

fn parse_default_derives(args: &mut Vec<TokenTree>) -> bool {
    // Extract the fields args and ensuring it is a key-value pair of Ident and Group
    take_ident_bool("default_derives", args).unwrap_or(true)
}

fn get_derive(
    defaults: bool,
    from_args: Vec<&Ident>,
    for_struct: bool,
) -> proc_macro2::TokenStream {
    let mut derives: Vec<proc_macro2::TokenStream> = vec![];

    if defaults {
        derives.push(quote!(Debug, Clone, PartialEq, Eq, PartialOrd, Ord));
        if for_struct {
            derives.push(quote!(Default));
            #[cfg(feature = "openapi")]
            {
                derives.push(quote!(::poem_openapi::Object));
            }
            #[cfg(feature = "builder")]
            {
                derives.push(quote!(::typed_builder::TypedBuilder));
            }
        }
    }
    if !from_args.is_empty() {
        derives.push(quote!(#(#from_args),*));
    }

    quote!(
        #[derive(#(#derives),*)]
    )
}
