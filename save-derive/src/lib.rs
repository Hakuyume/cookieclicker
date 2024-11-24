use proc_macro::TokenStream;
use quote::ToTokens;
use std::iter;
use syn::spanned::Spanned;

#[proc_macro_derive(Decode, attributes(decode))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match &input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => Generator {
            input: &input,
            fields,

            decode: syn::parse_quote!(__decode),
            error: syn::parse_quote!(__error),
            std: syn::parse_quote!(__std),
            tracing: syn::parse_quote!(__tracing),

            lifetime: syn::parse_quote!('__decode),
            segments: syn::parse_quote!(segments),
            value: syn::parse_quote!(value),
        }
        .generate()
        .into_token_stream()
        .into(),
        _ => quote::quote!(::core::compile_error!("unimplemented")).into(),
    }
}

struct Generator<'a> {
    input: &'a syn::DeriveInput,
    fields: &'a syn::FieldsNamed,

    decode: syn::Ident,
    error: syn::Ident,
    std: syn::Ident,
    tracing: syn::Ident,

    lifetime: syn::Lifetime,
    segments: syn::Ident,
    value: syn::Ident,
}

impl Generator<'_> {
    fn generate(&self) -> syn::Item {
        let Self {
            decode,
            error,
            std,
            tracing,
            lifetime,
            segments,
            value,
            ..
        } = self;
        let ident = &self.input.ident;

        let generics = self.genrics();
        let self_ty = self.self_ty();
        let where_predicates = into_compile_error(self.where_predicates());
        let segments_expr = self.segments_expr().map_or_else(
            syn::Error::into_compile_error,
            quote::ToTokens::into_token_stream,
        );
        let field_values = into_compile_error(self.field_values());

        syn::parse_quote!(
            const _: () = {
                use crate::decode as #decode;
                use crate::error as #error;
                use ::std as #std;
                use ::tracing as #tracing;

                impl<#(#generics,)*> #decode::Decoder<&#lifetime str, #self_ty> for #decode::Standard
                where
                #self_ty: #std::fmt::Debug,
                #(#where_predicates,)*
                {
                    #[#tracing::instrument(err, ret(level = #tracing::Level::DEBUG))]
                    fn decode(#value: &#lifetime str) -> Result<#self_ty, #error::Error> {
                        let mut #segments = #segments_expr;
                        Ok(#ident { #(#field_values,)* })
                    }
                }
            };
        )
    }

    fn genrics(&self) -> impl Iterator<Item = syn::GenericParam> + '_ {
        let Self { lifetime, .. } = self;

        iter::once(syn::parse_quote!(#lifetime)).chain(
            self.input.generics.params.iter().cloned().map(|mut param| {
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
            }),
        )
    }

    fn self_ty(&self) -> syn::Type {
        let ident = &self.input.ident;

        let generic_params =
            self.input
                .generics
                .params
                .iter()
                .map(|param| -> syn::GenericArgument {
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
                });
        syn::parse_quote!(#ident<#(#generic_params,)*>)
    }

    fn where_predicates(&self) -> impl Iterator<Item = syn::Result<syn::WherePredicate>> + '_ {
        let Self { decode, .. } = self;

        self.fields()
            .map(move |field| {
                let field = field?;
                let FieldNamed { ty, with, .. } = &field;

                let segment_ty = self.segment_ty()?;
                Ok(syn::parse_quote!(#with: #decode::Decoder<#segment_ty, #ty>))
            })
            .chain(
                self.input
                    .generics
                    .where_clause
                    .iter()
                    .flat_map(|where_clause| &where_clause.predicates)
                    .cloned()
                    .map(Ok),
            )
    }

    fn segments_expr(&self) -> syn::Result<syn::Expr> {
        let Self { value, .. } = self;

        if let Some(pat) = self.attrs()?.pat {
            Ok(syn::parse_quote!(#value.split(#pat)))
        } else {
            Ok(syn::parse_quote!(#value.chars()))
        }
    }

    fn segment_ty(&self) -> syn::Result<syn::Type> {
        let Self { lifetime, .. } = self;

        if self.attrs()?.pat.is_some() {
            Ok(syn::parse_quote!(&#lifetime str))
        } else {
            Ok(syn::parse_quote!(char))
        }
    }

    fn field_values(&self) -> impl Iterator<Item = syn::Result<syn::FieldValue>> + '_ {
        let Self {
            decode,
            error,
            std,
            tracing,
            segments,
            ..
        } = self;

        self.fields().map(move |field| {
            let field = field?;
            let FieldNamed {
                ident,
                ty,
                skip,
                with,
            } = &field;

            let segments: syn::Expr = if let Some(skip) = skip {
                syn::parse_quote!(#segments.by_ref().skip(#skip))
            } else {
                syn::parse_quote!(#segments)
            };
            let segment_ty = self.segment_ty()?;
            let decode: syn::TypePath =
                syn::parse_quote!(<#with as #decode::Decoder<#segment_ty, #ty>>::decode);
            Ok(syn::parse_quote!(
                #ident: {
                    let _span = #tracing::info_span!(#std::stringify!(#ident)).entered();
                    #decode(#segments.next().ok_or(#error::Error::InsufficientData)?)?
                }
            ))
        })
    }
}

struct Attrs {
    pat: Option<syn::LitChar>,
}
impl Generator<'_> {
    fn attrs(&self) -> syn::Result<Attrs> {
        let mut this = Attrs { pat: None };
        for attr in &self.input.attrs {
            if attr.path().is_ident("decode") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("pat") {
                        this.pat = Some(meta.value()?.parse()?);
                        Ok(())
                    } else {
                        Err(meta.error("unimplemented"))
                    }
                })?;
            }
        }
        Ok(this)
    }
}

struct FieldNamed<'a> {
    ident: &'a syn::Ident,
    ty: &'a syn::Type,
    skip: Option<syn::LitInt>,
    with: syn::Type,
}
impl Generator<'_> {
    fn fields(&self) -> impl Iterator<Item = syn::Result<FieldNamed<'_>>> + '_ {
        let Self { decode, .. } = self;

        self.fields.named.iter().map(move |field| {
            field.attrs.iter().try_fold(
                FieldNamed {
                    ident: field
                        .ident
                        .as_ref()
                        .ok_or_else(|| syn::Error::new(field.span(), "missing ident"))?,
                    ty: &field.ty,
                    skip: None,
                    with: syn::parse_quote!(#decode::Standard),
                },
                |mut this, attr| {
                    if attr.path().is_ident("decode") {
                        attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("skip") {
                                this.skip = Some(meta.value()?.parse()?);
                                Ok(())
                            } else if meta.path.is_ident("with") {
                                this.with = meta.value()?.parse()?;
                                Ok(())
                            } else {
                                Err(meta.error("unimplemented"))
                            }
                        })?
                    }
                    Ok(this)
                },
            )
        })
    }
}

fn into_compile_error<I, T>(iter: I) -> impl Iterator<Item = proc_macro2::TokenStream>
where
    I: IntoIterator<Item = syn::Result<T>>,
    T: quote::ToTokens,
{
    iter.into_iter().map(|item| match item {
        Ok(v) => v.into_token_stream(),
        Err(e) => e.into_compile_error(),
    })
}
