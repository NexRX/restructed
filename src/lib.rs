mod patch_model;
mod view_model;

use proc_macro::TokenStream;
use proc_macro2::{Group, Ident, TokenTree};
use quote::quote;
use syn::Attribute;

/// Creates a new struct akin to a patch model version of the original model. <br />
/// This is useful for scenarios partial data is needed (usually for updates). <br />
/// <br />
/// The new struct will have the same fields as the original model (unless epcified), but wrapped with: <br />
/// - [`Option`] if the field is *required* so **not** already a [`Option`]
/// - [`poem_openapi::types::MaybeUndefined`] if the field is *optional* so already a [`Option`]
/// <br/><br/>
/// # Requirements for Deriving
/// - The struct must derive [`poem_openapi::Object`]
/// <br/><br/>
/// # Arguments
/// Do note that Poem's OpenAPI are being hijacked here to provide configuration.<br/><br/>
/// ## Struct Level
/// All args are applied directly to the new struct.<br/><br/>
/// ## Field Level
/// All args are applied directly to the new struct's fields but there is one extra modification:
/// - `#[oai(read_only)]` - Omits the field from the new struct

#[proc_macro_derive(Models, attributes(view, patch))]
pub fn models(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let oai_attr = get_oai_attributes(&ast.attrs);

    let views: Vec<proc_macro2::TokenStream> = ast
        .attrs
        .iter()
        .filter(|v| is_attribute(v, "view"))
        .map(|a| view_model::impl_view_model_new(&ast, a, &oai_attr))
        .collect();

    let patches: Vec<proc_macro2::TokenStream> = ast
        .attrs
        .iter()
        .filter(|v| is_attribute(v, "patch"))
        .map(|a| patch_model::impl_patch_model_new(&ast, a, &oai_attr))
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

/// Gets the attribute name using [`get_attribute_name`] or panics if it doesn't exist in the expected form.
// fn unwrap_attribute_name(attr: &Attribute) -> &Ident {
//     match attr.meta.path().segments.first().map(|v| &v.ident) {
//         Some(v) => v,
//         None => panic!("First argument must be an identifier (name) of the struct for the view"),
//     }
// }

/// Extract a group for a given identifier, e.g. `name(...)`. The `(...)` part is returned.
fn get_ident_group<'a>(name: &str, args: &'a [TokenTree]) -> Option<&'a Group> {
    let mut group: Option<&Group> = None;
    for (i, tk) in args.iter().enumerate() {
        match tk {
            TokenTree::Ident(v) if *v == name => match args.get(i + 1) {
                Some(TokenTree::Group(g)) => group = Some(g),
                _ => panic!(
                    "Invalid or missing `{name}` argument, expected a group of args, e.g. `{name}(...)`"
                ),
            },
            _ => {}
        }
    }
    group
}

/// Parse the fields argument into a TokenStream, skipping checking for commas coz lazy
fn extract_idents(group: &Group) -> Vec<Ident> {
    group
        .stream()
        .into_iter()
        .filter_map(|tt| match tt {
            TokenTree::Ident(ident) => Some(ident),
            TokenTree::Punct(v) if v.as_char() == ',' => None,
            tt => panic!("Invalid syntax, expected a field identifier, got {}`", tt),
        })
        .collect()
}

/// Panics on unexpected args to show that they arent valid
fn panic_unexpected_args(names: Vec<&str>, args: &[TokenTree]) {
    for tk in args.iter() {
        match tk {
            TokenTree::Ident(v) if names.contains(&v.to_string().as_str()) => {}
            TokenTree::Ident(v) => {
                panic!(
                    "Unknown argument `{}`, all known arguments are {:?}",
                    v, names
                )
            }
            _ => {}
        }
    }
}
/// Parse a list of identifiers we want to derive. Will be empty if none are found.
fn parse_derives(args: &[TokenTree]) -> Vec<Ident> {
    // Extract the fields args and ensuring it is a key-value pair of Ident and Group
    let fields: &Group = match get_ident_group("derive", args) {
        Some(g) => g,
        None => return vec![],
    };

    extract_idents(fields)
}

fn get_derive(defaults: bool, from_args: Vec<&Ident>) -> proc_macro2::TokenStream {
    let mut derives: Vec<proc_macro2::TokenStream> = vec![];

    if defaults {
        derives.push(quote!(
            Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord
        ));
    }
    if !from_args.is_empty() {
        derives.push(quote!(#(#from_args),*));
    }
    #[cfg(feature = "openapi")]
    {
        derives.push(quote!(::poem_openapi::Object));
    }
    #[cfg(feature = "builder")]
    {
        derives.push(quote!(::typed_builder::TypedBuilder));
    }

    quote!(
        #[derive(#(#derives),*)]
    )
}
