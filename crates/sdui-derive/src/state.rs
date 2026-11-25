use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Expr, ExprLit, Fields, Lit, Meta, Result, Token,
    punctuated::Punctuated, spanned::Spanned,
};

pub fn derive(input: TokenStream) -> Result<TokenStream> {
    let input = syn::parse2::<DeriveInput>(input)?;
    if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
        return Err(syn::Error::new(
            input.generics.span(),
            "A2uiData does not support generic state",
        ));
    }

    match input.data {
        Data::Enum(_) => Ok(leaf(&input.ident)),
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => structure(&input.attrs, &input.vis, &input.ident, &fields),
            _ => Err(syn::Error::new(
                input.ident.span(),
                "A2uiData requires a struct with named fields or an enum",
            )),
        },
        Data::Union(_) => Err(syn::Error::new(
            input.ident.span(),
            "A2uiData cannot be derived for unions",
        )),
    }
}

fn leaf(ident: &syn::Ident) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl ::sdui::A2uiData for #ident {
            type Path = ::a2ui::DataBinding;

            fn from_path(path: ::sdui::StatePath) -> Self::Path {
                ::a2ui::DataBinding::new(path.into_string())
            }
        }
    }
}

fn structure(
    attrs: &[Attribute],
    vis: &syn::Visibility,
    ident: &syn::Ident,
    fields: &syn::FieldsNamed,
) -> Result<TokenStream> {
    let rename_all = container_rename_all(attrs)?;
    let paths = format_ident!("{ident}Paths");
    let methods = fields
        .named
        .iter()
        .map(|field| {
            let rules = field_rules(&field.attrs)?;
            if rules.skip {
                return Ok(None);
            }
            let field_ident = field.ident.as_ref().expect("named field");
            let field_type = &field.ty;
            let field_vis = &field.vis;
            let path = if rules.flatten {
                quote!(self.path.clone())
            } else {
                let name = rules.rename.unwrap_or_else(|| {
                    rename_all.as_deref().map_or_else(
                        || field_name(field_ident),
                        |rule| apply(rule, &field_name(field_ident)),
                    )
                });
                quote!(self.path.child(#name))
            };
            Ok(Some(quote! {
                #[must_use]
                #field_vis fn #field_ident(&self) -> <#field_type as ::sdui::A2uiData>::Path {
                    <#field_type as ::sdui::A2uiData>::from_path(#path)
                }
            }))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        #[derive(Clone, Debug)]
        #vis struct #paths {
            path: ::sdui::StatePath,
        }

        #[automatically_derived]
        impl ::sdui::A2uiData for #ident {
            type Path = #paths;

            fn from_path(path: ::sdui::StatePath) -> Self::Path {
                #paths { path }
            }
        }

        impl #ident {
            #[must_use]
            #vis fn paths() -> #paths {
                <Self as ::sdui::A2uiData>::from_path(::sdui::StatePath::root())
            }
        }

        impl #paths {
            #(#methods)*
        }
    })
}

#[derive(Default)]
struct FieldRules {
    rename: Option<String>,
    skip: bool,
    flatten: bool,
}

fn field_rules(attrs: &[Attribute]) -> Result<FieldRules> {
    let mut rules = FieldRules::default();
    for meta in serde_meta(attrs)? {
        match meta {
            Meta::NameValue(value) if value.path.is_ident("rename") => {
                rules.rename = Some(string_value(value.value)?);
            }
            Meta::Path(path) if path.is_ident("skip") || path.is_ident("skip_serializing") => {
                rules.skip = true;
            }
            Meta::Path(path) if path.is_ident("flatten") => rules.flatten = true,
            _ => {}
        }
    }
    Ok(rules)
}

fn container_rename_all(attrs: &[Attribute]) -> Result<Option<String>> {
    let mut rename_all = None;
    for meta in serde_meta(attrs)? {
        if let Meta::NameValue(value) = meta
            && value.path.is_ident("rename_all")
        {
            rename_all = Some(string_value(value.value)?);
        }
    }
    Ok(rename_all)
}

fn serde_meta(attrs: &[Attribute]) -> Result<Vec<Meta>> {
    let mut output = Vec::new();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
        let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        output.extend(nested);
    }
    Ok(output)
}

fn string_value(value: Expr) -> Result<String> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(value),
        ..
    }) = value
    {
        Ok(value.value())
    } else {
        Err(syn::Error::new(value.span(), "expected a string literal"))
    }
}

fn field_name(ident: &syn::Ident) -> String {
    ident
        .to_string()
        .strip_prefix("r#")
        .map_or_else(|| ident.to_string(), str::to_owned)
}

fn apply(rule: &str, field: &str) -> String {
    match rule {
        "PascalCase" => pascal(field),
        "camelCase" => {
            let value = pascal(field);
            let mut characters = value.chars();
            let first = characters
                .next()
                .map_or_default(|character| character.to_lowercase().collect::<String>());
            format!("{first}{}", characters.as_str())
        }
        "UPPERCASE" | "SCREAMING_SNAKE_CASE" => field.to_ascii_uppercase(),
        "kebab-case" => field.replace('_', "-"),
        "SCREAMING-KEBAB-CASE" => field.to_ascii_uppercase().replace('_', "-"),
        _ => field.to_owned(),
    }
}

fn pascal(field: &str) -> String {
    let mut output = String::new();
    let mut capitalize = true;
    for character in field.chars() {
        if character == '_' {
            capitalize = true;
        } else if capitalize {
            output.extend(character.to_uppercase());
            capitalize = false;
        } else {
            output.push(character);
        }
    }
    output
}
