use super::{
    abort_unexpected_args, extract_idents, has_oai_attribute, take_ident_group, take_ident_ident,
    take_ident_literal, take_path_group,
};
use proc_macro2::{Ident, TokenTree};
use proc_macro_error::abort;
use syn::{Attribute, Field};

#[derive(Clone, Default)]
pub(crate) struct ModelAttrArgs {
    pub base: Option<BaseAttrArgs>,
    pub defaults: Option<DefaultAttrArgs>,
    pub extras: ExtraConditions,
}

impl ModelAttrArgs {
    /// Conditional aborts on unexpected args to show that they arent valid
    pub(crate) fn abort_unexpected(args: &[TokenTree]) {
        const EXPECTED: &[&str; 2] = &["base", "defaults"];
        abort_unexpected_args(EXPECTED.to_vec(), args);
    }

    pub(crate) fn parse(attrs: Vec<Attribute>, extras: ExtraConditions) -> Self {
        if attrs.is_empty() {
            return Self::default();
        } else if attrs.len() > 1 {
            abort!(
                attrs[1],
                "Invalid attribute, expected only one `model` attribute but got `{}`",
                attrs.len()
            )
        }

        let attr = attrs.first().unwrap();

        let mut args: Vec<TokenTree> = attr
            .meta
            .require_list()
            .expect("This attribute must be in a list format")
            .to_owned()
            .tokens
            .into_iter()
            .collect::<Vec<_>>();
        let args_mr = &mut args;

        let base = take_ident_group("base", args_mr)
            .map(|g| BaseAttrArgs::parse(&mut g.stream().into_iter().collect(), attr));

        let defaults = take_ident_group("defaults", args_mr)
            .map(|g| DefaultAttrArgs::parse(&mut g.stream().into_iter().collect(), attr));

        Self::abort_unexpected(&args);

        Self {
            base,
            defaults,
            extras,
        }
    }
}

#[derive(Clone)]
pub(crate) struct BaseAttrArgs {
    pub fields: Option<FieldsArg>,
    pub derive: Option<Vec<syn::Path>>,
}

impl BaseAttrArgs {
    /// Parses the attribute and returns the parsed arguments (0) and any remaining (1)
    pub(crate) fn parse(args: &mut Vec<TokenTree>, attr: &Attribute) -> Self {
        let fields = {
            let fields = FieldsArg::parse(args, attr);
            match fields.is_default() {
                true => None,
                false => Some(fields),
            }
        };
        let derive = take_path_group("derive", args);

        Self { fields, derive }
    }
}

#[derive(Clone)]
pub(crate) struct DefaultAttrArgs {
    pub fields: Option<FieldsArg>,
    pub derive: Option<Vec<syn::Path>>,
    pub preset: Option<Preset>,
    pub attributes_with: AttributesWith, // Has it's own None
}

impl DefaultAttrArgs {
    /// Parses the attribute and returns the parsed arguments (0) and any remaining (1)
    pub(crate) fn parse(args: &mut Vec<TokenTree>, attr: &Attribute) -> Self {
        let fields = {
            let fields = FieldsArg::parse(args, attr);
            match fields.is_default() {
                true => None,
                false => Some(fields),
            }
        };
        let derive = take_path_group("derive", args);
        let preset = Preset::parse(args);
        let attributes_with = AttributesWith::parse(args).unwrap_or_else(|| match preset {
            Some(p) => p.attr_with(),
            None => AttributesWith::None,
        });

        Self {
            fields,
            derive,
            preset,
            attributes_with,
        }
    }
}

#[derive(Clone)]
pub(crate) struct AttrArgs {
    /// The name of the newly derived struct
    pub name: Ident,
    /// Fields the be generated via omitting or listing
    pub fields: FieldsArg,
    /// Things to derive on the derived struct
    pub derive: Option<Vec<syn::Path>>,
    /// Presets to apply to the derived struct
    pub preset: Preset,
    /// Attributes to carry over onto the derived struct
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

    /// Parses the attribute and returns the parsed arguments as `Self` (0) and any arguments remaining unparsed (1)
    pub(crate) fn parse(
        attr: &Attribute,
        model_args: &ModelAttrArgs,
        abort_unexpected: bool,
    ) -> (Self, Vec<TokenTree>) {
        let tks: Vec<TokenTree> = attr
            .meta
            .require_list()
            .expect("This attribute must be in a list format")
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

        let mut args = match tks.len() < 3 {
            true => vec![],
            false => tks[2..].to_vec(),
        };

        let fields = FieldsArg::parse_with_args(&mut args, model_args, attr);
        let derive = parse_derives_wtih_args(&mut args, model_args);
        let preset = Preset::parse_with_args(&mut args, model_args);
        let attributes_with =
            AttributesWith::parse_with_args(&mut args, model_args, preset.as_ref())
                .unwrap_or_default();

        if abort_unexpected {
            Self::abort_unexpected(&args, &[])
        }

        (
            Self {
                name,
                fields,
                derive,
                preset: preset.unwrap_or_default(),
                attributes_with,
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

        // Parse the fields argument into a TokenStream, skip checking for commas coz lazy
        match (field_arg, omit_args) {
            (Some(g), None) => Self::Fields(extract_idents(g)),
            (None, Some(g)) => Self::Omit(extract_idents(g)),
            (None, None) => Self::default(),
            (Some(_), Some(_)) => abort!(
                attr_spanned,
                "Cannot have both `fields` and `omit` arguments"
            ),
        }
    }

    pub(crate) fn parse_with_args(
        args: &mut Vec<TokenTree>,
        model_args: &ModelAttrArgs,
        attr: &Attribute, // Just for its span and error highlighting purposes
    ) -> Self {
        use FieldsArg::*;
        let fields = FieldsArg::parse(args, attr);

        let default_fields = model_args.defaults.as_ref().and_then(|v| v.fields.clone());
        let fields = match &default_fields {
            Some(f) if fields.is_default() => f.clone(),
            _ => fields,
        };

        let base_fields = model_args.base.as_ref().and_then(|v| v.fields.clone());
        if let Some(base) = base_fields {
            let final_fields: Vec<_> = match (fields, base) {
                (Fields(f), Fields(ref b)) => f
                    .into_iter()
                    .filter(|v| !b.contains(v))
                    .chain(b.clone())
                    .collect(),
                (Fields(f), Omit(b)) => f.into_iter().filter(|v| !b.contains(v)).collect(),
                (Omit(f), Fields(b)) => b.into_iter().filter(|v| !f.contains(v)).collect(),
                (Omit(f), Omit(mut b)) => {
                    let not_in_base = f.into_iter().filter(|v| !b.contains(v)).collect::<Vec<_>>();
                    b.extend(not_in_base);
                    b
                }
            };
            Fields(final_fields)
        } else {
            fields
        }
    }

    /// Similar to an is_empty function but only checks if omit is empty as thats the default case
    pub(crate) fn is_default(&self) -> bool {
        match self {
            Self::Omit(fields) => fields.is_empty(),
            _ => false,
        }
    }

    pub(crate) fn predicate(&self, field: &Ident) -> bool {
        match self {
            Self::Fields(fields) => fields.contains(field),
            Self::Omit(fields) => !fields.contains(field),
        }
    }
}

impl Default for FieldsArg {
    fn default() -> Self {
        Self::Omit(vec![])
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

    pub(crate) fn parse_with_args(
        args: &mut Vec<TokenTree>,
        model_args: &ModelAttrArgs,
        preset: Option<&Preset>,
    ) -> Option<Self> {
        AttributesWith::parse(args).or_else(|| match preset {
            Some(preset) => Some(preset.attr_with()),
            None => model_args.defaults.as_ref().map(|f| f.attributes_with),
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
    pub(crate) fn parse_with_args(
        args: &mut Vec<TokenTree>,
        model_args: &ModelAttrArgs,
    ) -> Option<Self> {
        Preset::parse(args).or(model_args.defaults.as_ref().and_then(|f| f.preset))
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

/// Parses the `derive` attribute and returns the parsed arguments as a Vec of `syn::Path` if the argument was given
fn parse_derives_wtih_args(
    args: &mut Vec<TokenTree>,
    model_args: &ModelAttrArgs,
) -> Option<Vec<syn::Path>> {
    let base_derives = model_args.base.as_ref().and_then(|v| v.derive.clone());
    let default_derives = model_args.defaults.as_ref().and_then(|v| v.derive.clone());

    let derives = take_path_group("derive", args).or(default_derives.clone());
    match (derives, base_derives) {
        (Some(d), Some(mut b)) => {
            b.extend(d);
            Some(b)
        }
        (Some(g), None) => Some(g),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

#[derive(Clone, Default)]
pub(crate) struct ExtraConditions {
    #[cfg(feature = "openapi")]
    /// When true, the deriving struct has an `#[oai(example)]` attribute and the derived struct should handle this.
    pub has_oai_example: bool,
}

impl ExtraConditions {
    pub(crate) fn parse(attr: &[Attribute]) -> Self {
        Self {
            #[cfg(feature = "openapi")]
            has_oai_example: has_oai_attribute(attr, Some("example")),
        }
    }
}
