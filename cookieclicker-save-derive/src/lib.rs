use proc_macro::TokenStream;
use quote::ToTokens;
use std::iter;
use syn::spanned::Spanned;

#[proc_macro_derive(Format, attributes(format))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as Input);
    derive_impl(&input).into_token_stream().into()
}

fn derive_impl(input: &Input) -> syn::Item {
    let impl_generics = input.impl_generics();
    let ty_generics = input.ty_generics().collect::<Vec<_>>();
    let where_predicates = input.where_predicates();

    let with_default = syn::parse_quote!(__format::Standard);

    match input {
        Input::StructNamed {
            split,
            trailing,
            ident,
            fields,
            ..
        } => {
            let where_predicates =
                where_predicates.chain(fields.iter().map(|FieldNamed { with, ty, .. }| {
                    let with = with.as_ref().unwrap_or(&with_default);
                    syn::parse_quote!(#with: __format::Format<'__format, #ty>)
                }));

            let decode_split: syn::Expr = if let Some(split) = split {
                syn::parse_quote!(value.split(#split))
            } else {
                syn::parse_quote!(__format::chars(value))
            };
            let decode_field_values = fields.iter().map(
                |FieldNamed { with, ident, ty }| -> syn::FieldValue {
                    let with = with.as_ref().unwrap_or(&with_default);
                    syn::parse_quote!(
                        #ident: {
                            let _span = __tracing::info_span!(__std::stringify!(#ident)).entered();
                            <#with as __format::Format<'__format, #ty>>::decode(
                                split.next().ok_or(__error::Error::InsufficientData)?
                            )?
                        }
                    )
                },
            );

            let encode_exprs =
                fields
                    .iter()
                    .enumerate()
                    .flat_map(|(i, FieldNamed { with, ident, ty })| {
                        let with = with.as_ref().unwrap_or(&with_default);
                        iter::once(syn::parse_quote!(
                            <#with as __format::Format<'__format, #ty>>::encode(&value.#ident, f)
                        ))
                        .chain(
                            split
                                .iter()
                                .filter(move |_| {
                                    i + 1 < fields.len()
                                        || trailing
                                            .as_ref()
                                            .map(syn::LitBool::value)
                                            .unwrap_or(false)
                                })
                                .map(|split| -> syn::Expr {
                                    syn::parse_quote!(__fmt::Display::fmt(&#split, f))
                                }),
                        )
                    });

            let check_inverse_hook_blocks = fields.iter().map(
                |FieldNamed { with, ident, ty }| -> syn::Block {
                    let with = with.as_ref().unwrap_or(&with_default);
                    syn::parse_quote!(
                        {
                            let _span = __tracing::info_span!(__std::stringify!(#ident)).entered();
                            __format::check_inverse::<'__format, '__check_inverse_hook, #with, #ty>(
                                split.next().ok_or(__error::Error::InsufficientData)?
                            )?;
                        }
                    )
                },
            );

            syn::parse_quote!(
                const _: () = {
                    use crate::error as __error;
                    use crate::format as __format;
                    #[cfg(test)]
                    use ::anyhow as __anyhow;
                    use ::std as __std;
                    use ::std::fmt as __fmt;
                    use ::tracing as __tracing;

                    impl<'__format, #(#impl_generics,)*> __format::Format<'__format, #ident<#(#ty_generics,)*>> for __format::Standard
                    where
                    #(#where_predicates,)*
                    {
                        #[__tracing::instrument(err)]
                        fn decode(value: &'__format str) -> Result<#ident<#(#ty_generics,)*>, __error::Error> {
                            let mut split = #decode_split;
                            Ok(#ident {#(#decode_field_values,)*})
                        }

                        fn encode(value: &#ident<#(#ty_generics,)*>, f: &mut __fmt::Formatter<'_>) -> __fmt::Result {
                            #(#encode_exprs?;)*
                            Ok(())
                        }

                        #[cfg(test)]
                        #[__tracing::instrument(err)]
                        fn check_inverse_hook<'__check_inverse_hook>(value: &'__check_inverse_hook str) -> __anyhow::Result<()>
                        where
                        '__check_inverse_hook: '__format,
                        Self: '__check_inverse_hook,
                        {
                            let mut split = #decode_split;
                            #(#check_inverse_hook_blocks)*
                            Ok(())
                        }
                    }
                };
            )
        }
    }
}

enum Input {
    StructNamed {
        split: Option<syn::Expr>,
        trailing: Option<syn::LitBool>,

        ident: syn::Ident,
        generics: syn::Generics,
        fields: Vec<FieldNamed>,
    },
}

struct FieldNamed {
    with: Option<syn::Type>,

    ident: syn::Ident,
    ty: syn::Type,
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let input = syn::DeriveInput::parse(input)?;

        let mut split = None;
        let mut trailing = None;
        for attr in &input.attrs {
            if attr.path().is_ident("format") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("split") {
                        split = Some(meta.value()?.parse()?);
                        Ok(())
                    } else if meta.path.is_ident("trailing") {
                        trailing = Some(meta.value()?.parse()?);
                        Ok(())
                    } else {
                        Err(meta.error("unknown"))
                    }
                })?;
            }
        }

        match input.data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Named(fields),
                ..
            }) => {
                let fields = fields
                    .named
                    .into_iter()
                    .map(|field| {
                        let span = field.span();

                        let mut with = None;
                        for attr in &field.attrs {
                            if attr.path().is_ident("format") {
                                attr.parse_nested_meta(|meta| {
                                    if meta.path.is_ident("with") {
                                        with = Some(meta.value()?.parse()?);
                                        Ok(())
                                    } else {
                                        Err(meta.error("unknown"))
                                    }
                                })?;
                            }
                        }

                        Ok(FieldNamed {
                            with,
                            ident: field.ident.ok_or(syn::Error::new(span, "missing ident"))?,
                            ty: field.ty,
                        })
                    })
                    .collect::<syn::Result<_>>()?;
                Ok(Self::StructNamed {
                    split,
                    trailing,
                    ident: input.ident,
                    generics: input.generics,
                    fields,
                })
            }
            _ => Err(syn::Error::new(input.span(), "unimplemented")),
        }
    }
}

impl Input {
    fn impl_generics(&self) -> impl Iterator<Item = syn::GenericParam> + '_ {
        let Self::StructNamed { generics, .. } = self;
        generics.params.iter().cloned().map(|mut param| {
            match &mut param {
                syn::GenericParam::Lifetime(param) => {
                    param.attrs.clear();
                }
                syn::GenericParam::Type(param) => {
                    param.eq_token = None;
                    param.default = None;
                }
                syn::GenericParam::Const(param) => {
                    param.eq_token = None;
                    param.default = None;
                }
            }
            param
        })
    }

    fn ty_generics(&self) -> impl Iterator<Item = syn::GenericArgument> + '_ {
        let Self::StructNamed { generics, .. } = self;
        generics.params.iter().map(|param| -> syn::GenericArgument {
            match param {
                syn::GenericParam::Lifetime(param) => {
                    let lifetime = &param.lifetime;
                    syn::parse_quote!(#lifetime)
                }
                syn::GenericParam::Type(param) => {
                    let ident = &param.ident;
                    syn::parse_quote!(#ident)
                }
                syn::GenericParam::Const(param) => {
                    let ident = &param.ident;
                    syn::parse_quote!(#ident)
                }
            }
        })
    }

    fn where_predicates(&self) -> impl Iterator<Item = syn::WherePredicate> + '_ {
        let Self::StructNamed { generics, .. } = self;
        generics
            .where_clause
            .iter()
            .flat_map(|where_clause| &where_clause.predicates)
            .cloned()
    }
}
