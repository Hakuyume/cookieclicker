mod decode;
mod encode;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::spanned::Spanned;

#[proc_macro_derive(Encode, attributes(format))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as Input);
    encode::derive(&input).into_token_stream().into()
}

#[proc_macro_derive(Decode, attributes(format))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as Input);
    decode::derive(&input).into_token_stream().into()
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
    skip: Option<syn::Expr>,

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
                        let mut skip = None;
                        for attr in &field.attrs {
                            if attr.path().is_ident("format") {
                                attr.parse_nested_meta(|meta| {
                                    if meta.path.is_ident("as") {
                                        as_ = Some(meta.value()?.parse()?);
                                        Ok(())
                                    } else if meta.path.is_ident("skip") {
                                        skip = Some(meta.value()?.parse()?);
                                        Ok(())
                                    } else {
                                        Err(meta.error("unknown"))
                                    }
                                })?;
                            }
                        }

                        Ok(FieldNamed {
                            as_,
                            skip,

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
