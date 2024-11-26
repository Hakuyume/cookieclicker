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
    let ty_generics = input.ty_generics();
    let where_predicates = input.where_predicates();

    let as_default = syn::parse_quote!(__format::Same);

    match input {
        Input::StructNamed {
            split,
            ident,
            fields,
            ..
        } => {
            let where_predicates =
                where_predicates.chain(fields.iter().map(|FieldNamed { as_, ty, .. }| {
                    let as_ = as_.as_ref().unwrap_or(&as_default);
                    syn::parse_quote!(#as_: __format::FormatAs<'__format, #ty>)
                }));

            let decode_split: syn::Expr = if let Some(split) = split {
                syn::parse_quote!(value.split(#split))
            } else {
                syn::parse_quote!(__format::chars(value))
            };
            let decode_field_values = fields.iter().map(
                |FieldNamed { as_, ident, ty }| -> syn::FieldValue {
                    let as_ = as_.as_ref().unwrap_or(&as_default);
                    syn::parse_quote!(
                        #ident: {
                            let _span = __tracing::info_span!(__std::stringify!(#ident)).entered();
                            <#as_ as __format::FormatAs<'__format, #ty>>::decode_as(
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
                    .flat_map(|(i, FieldNamed { as_, ident, ty })| {
                        let as_ = as_.as_ref().unwrap_or(&as_default);
                        split
                        .iter()
                        .filter(move |_| i > 0)
                        .map(|split| -> syn::Expr {
                            syn::parse_quote!(__fmt::Display::fmt(&#split, f))
                        })
                        .chain(iter::once(syn::parse_quote!(
                            <#as_ as __format::FormatAs<'__format, #ty>>::encode_as(&self.#ident, f)
                        )))
                    });

            syn::parse_quote!(
                const _: () = {
                    use crate::error as __error;
                    use crate::format as __format;
                    use ::std as __std;
                    use ::std::fmt as __fmt;
                    use ::tracing as __tracing;

                    impl <'__format, #(#impl_generics,)*> __format::Format<'__format> for #ident<#(#ty_generics,)*>
                    where
                    #(#where_predicates,)*
                    {
                        #[__tracing::instrument(err)]
                        fn decode(value: &'__format str) -> Result<Self, __error::Error> {
                            let mut split = #decode_split;
                            Ok(Self {#(#decode_field_values,)*})
                        }

                        fn encode(&self, f: &mut __fmt::Formatter<'_>) -> __fmt::Result {
                            #(#encode_exprs?;)*
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

        ident: syn::Ident,
        generics: syn::Generics,
        fields: Vec<FieldNamed>,
    },
}

struct FieldNamed {
    as_: Option<syn::Type>,

    ident: syn::Ident,
    ty: syn::Type,
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let input = syn::DeriveInput::parse(input)?;

        let mut split = None;
        for attr in &input.attrs {
            if attr.path().is_ident("format") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("split") {
                        split = Some(meta.value()?.parse()?);
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

                        let mut as_ = None;
                        for attr in &field.attrs {
                            if attr.path().is_ident("format") {
                                attr.parse_nested_meta(|meta| {
                                    if meta.path.is_ident("as") {
                                        as_ = Some(meta.value()?.parse()?);
                                        Ok(())
                                    } else {
                                        Err(meta.error("unknown"))
                                    }
                                })?;
                            }
                        }

                        Ok(FieldNamed {
                            as_,
                            ident: field.ident.ok_or(syn::Error::new(span, "missing ident"))?,
                            ty: field.ty,
                        })
                    })
                    .collect::<syn::Result<_>>()?;
                Ok(Self::StructNamed {
                    split,
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
