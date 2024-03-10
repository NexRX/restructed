use super::{
    abort_unexpected_args, extract_idents, has_oai_attribute, take_ident_group, take_ident_ident, take_ident_literal, take_path_group
};
use proc_macro2::{Ident, TokenTree};
use proc_macro_error::abort;
use syn::{Attribute, Field};

#[derive(Clone)]
pub(crate) struct AttrArgs {
    pub name: Ident,
    pub fields: FieldsArg,
    pub derive: Option<Vec<syn::Path>>,
    pub preset: Preset,
    pub attributes_with: AttributesWith,
}

impl AttrArgs {
    /// Conditional aborts on unexpected args to show that they arent valid
    pub(crate) fn abort_unexpected(args: &[TokenTree], ignore: &[&str]) {
        const EXPECTED: &[&str; 5] = &["fields", "omit", "derive", "attributes_with", "preset"];
        let mut expect = EXPECTED.to_vec();
        expect.extend(ignore);
        abort_unexpected_args(expect, args)
    }

    /// Parses the attribute and returns the parsed arguments (0) and any remaining (1)
    pub(crate) fn parse(attr: &Attribute, abort_unexpected: bool) -> (Self, Vec<TokenTree>) {
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

        let mut args = tks[2..].to_vec();
        let args_mr = &mut args;

        // Parse Expected Macro Args
        let fields = FieldsArg::parse(args_mr, attr);
        let derive = take_path_group("derive", args_mr);
        let preset = Preset::parse(args_mr).unwrap_or_default();
        let attributes_with = AttributesWith::parse(args_mr).unwrap_or_else(|| preset.attr_with());
        
        if abort_unexpected {
            Self::abort_unexpected(&args, &[])
        }

        (
            Self {
                name,
                fields,
                derive,
                preset,
                attributes_with
            },
            args,
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) enum FieldsArg {
    /// Fields to be included in the model (A whitelist)
    Fields(Vec<Ident>),
    /// Fields to be omitted from the model (A blacklist)
    Omit(Vec<Ident>),
}

impl FieldsArg {
    pub(crate) fn parse(args: &mut Vec<TokenTree>, attr_spanned: &Attribute) -> Self {
        // Extract the fields args and ensuring it is a key-value pair of Ident and Group

        let field_arg = take_ident_group("fields", args);
        let omit_args = take_ident_group("omit", args);

        if field_arg.is_some() && omit_args.is_some() {
            abort!(
                attr_spanned,
                "Cannot have both `fields` and `omit` arguments"
            )
        }

        // Parse the fields argument into a TokenStream, skip checking for commas coz lazy
        match (field_arg, omit_args) {
            (Some(g), None) => Self::Fields(extract_idents(g)),
            (None, Some(g)) => Self::Omit(extract_idents(g)),
            (None, None) => Self::Omit(vec![]),
            (Some(_), Some(_)) => abort!(
                attr_spanned,
                "Cannot have both `fields` and `omit` arguments"
            ),
        }
    }

    pub(crate) fn predicate(&self, field: &Ident) -> bool {
        match self {
            Self::Fields(fields) => fields.contains(field),
            Self::Omit(fields) => !fields.contains(field),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum AttributesWith {
    #[default]
    None,
    #[cfg(feature = "openapi")]
    Oai,
    Deriveless,
    All,
}

impl AttributesWith {
    pub(crate) fn parse(args: &mut Vec<TokenTree>) -> Option<Self> {
        let ident = take_ident_literal("attributes_with", args)?;

        let value = ident.to_string();
        Some(match &value[1..value.chars().count() - 1] {
            "none" => Self::None,
            #[cfg(feature = "openapi")] "oai" => Self::Oai,
            "deriveless" => Self::Deriveless,
            "all" => Self::All,
            #[cfg(feature = "openapi")]
            v => abort!(
                    ident,
                    "Invalid value, expected `none`, `oai` (from poem_openapi crate), `deriveless`, or `all` but got `{}`", v
                ),
            #[cfg(not(feature = "openapi"))]
            v => abort!(
                    ident,
                    "Invalid value, expected `none`, `deriveless`, or `all` but got `{}`", v
                ),
        })
    }

    pub(crate) fn gen_top_attributes<'a>(&self, ast: &'a syn::DeriveInput) -> Vec<&'a Attribute> {
        match self {
            Self::All => ast
                .attrs
                .iter()
                .filter(|attr| {
                    attr.path().segments.first().map_or(true, |seg| {
                        !matches!(seg.ident.to_string().as_str(), "view" | "patch")
                    }) // update if we add more
                })
                .collect(),
            Self::Deriveless => ast
                .attrs
                .iter()
                .filter(|attr| {
                    attr.path().segments.first().map_or(false, |seg| {
                        !matches!(seg.ident.to_string().as_str(), "view" | "patch" | "derive")
                    }) // update if we add more
                })
                .collect(),
            #[cfg(feature = "openapi")]
            Self::Oai => ast
                .attrs
                .iter()
                .filter(|attr| attr.meta.path().is_ident("oai"))
                .collect(),
            Self::None => vec![],
        }
    }

    pub(crate) fn gen_field_attributes(&self, attrs: Vec<Attribute>) -> Vec<Attribute> {
        match self {
            #[cfg(feature = "openapi")]
            Self::Oai => attrs
                .into_iter()
                .filter(|attr| {
                    attr.meta
                        .path()
                        .segments
                        .first()
                        .map_or(false, |seg| seg.ident == "oai")
                })
                .collect::<Vec<_>>(),
            Self::All | AttributesWith::Deriveless => attrs.into_iter().collect::<Vec<_>>(), // change if we ever add field level attributes to this crate
            Self::None => vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum OptionType {
    #[default]
    Option,
    MaybeUndefined,
}

impl OptionType {
    pub(crate) fn parse(args: &mut Vec<TokenTree>) -> Option<OptionType> {
        let ident = take_ident_ident("option", args)?;

        Some(match ident.to_string().as_str() {
            "Option" => OptionType::Option,
            "MaybeUndefined" => OptionType::MaybeUndefined,
            _ => abort!(
                ident,
                "Invalid type, expected `Option` or `MaybeUndefined` (from poem_openapi crate)"
            ),
        })
    }
    
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum Preset {
    #[default]
    None,
    #[cfg(feature = "openapi")]
    Read,
    #[cfg(feature = "openapi")]
    Write,
}

impl Preset {
    pub(crate) fn parse(args: &mut Vec<TokenTree>) -> Option<Self> {
        let ident = take_ident_literal("preset", args)?;

        let value = ident.to_string();
        Some(match &value[1..value.chars().count() - 1] {
            "none" => Self::None,
            #[cfg(feature = "openapi")]
            "read" => Self::Read,
            #[cfg(feature = "openapi")]
            "write" => Self::Write,
            #[cfg(feature = "openapi")]
            v => abort!(
                    ident,
                    "Invalid value, expected `none` or `read`/`write` (with `openapi` feature) but got `{}`", v
                ),
            #[cfg(not(feature = "openapi"))]
            v => abort!(
                    ident,
                    "Invalid value, expected `none` but got `{}`", v
                ),
        })
    }

    pub(crate) fn predicate(&self, field: &Field) -> bool {
        match self {
            Self::None => true,
            #[cfg(feature = "openapi")]
            Self::Read => !has_oai_attribute(&field.attrs, Some("write_only")),
            #[cfg(feature = "openapi")]
            Self::Write => !has_oai_attribute(&field.attrs, Some("read_only")),
        }
    }

    pub(crate) fn option(&self) -> OptionType {
        match self {
            Self::None => OptionType::Option,
            #[cfg(feature = "openapi")]
            Self::Read | Self::Write => OptionType::MaybeUndefined,
        }
    }

    pub(crate) fn attr_with(&self) -> AttributesWith {
        match self {
            Self::None => AttributesWith::None,
            #[cfg(feature = "openapi")]
            Self::Read | Self::Write => AttributesWith::Oai,
        }
    }
}
