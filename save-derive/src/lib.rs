use proc_macro::TokenStream;
use quote::ToTokens;
use std::iter;
use syn::spanned::Spanned;

#[proc_macro_derive(Decode, attributes(format))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match &input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => Generator {
            input: &input,
            fields,

            error: syn::parse_quote!(__error),
            format: syn::parse_quote!(__format),
            std: syn::parse_quote!(__std),
            tracing: syn::parse_quote!(__tracing),

            lifetime: syn::parse_quote!('__decode),
            split: syn::parse_quote!(split),
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

    error: syn::Ident,
    format: syn::Ident,
    std: syn::Ident,
    tracing: syn::Ident,

    lifetime: syn::Lifetime,
    split: syn::Ident,
    value: syn::Ident,
}

impl Generator<'_> {
    fn generate(&self) -> syn::Item {
        let Self {
            error,
            format,
            std,
            tracing,
            lifetime,
            split,
            value,
            ..
        } = self;
        let generics = self.genrics();
        let self_ty = self.self_ty();
        let where_predicates = into_compile_error(self.where_predicates());
        let split_pat = self
            .attrs()
            .map_or_else(syn::Error::into_compile_error, |attrs| {
                attrs.split.into_token_stream()
            });
        let field_values = into_compile_error(self.field_values());

        syn::parse_quote!(
            const _: () = {
                use crate::error as #error;
                use crate::format as #format;
                use ::std as #std;
                use ::tracing as #tracing;

                impl<#(#generics,)*> #format::Decode<#lifetime> for #self_ty
                where
                #(#where_predicates,)*
                {
                    fn decode(#value: &#lifetime str) -> Result<Self, #error::Error> {
                        let mut #split = #value.split(#split_pat);
                        Ok(Self { #(#field_values,)* })
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
        let Self {
            format, lifetime, ..
        } = self;

        self.fields()
            .map(move |field| {
                let field = field?;
                let FieldNamed { ty, as_, .. } = &field;

                Ok(syn::parse_quote!(#as_: #format::DecodeAs<#lifetime, #ty>))
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

    fn field_values(&self) -> impl Iterator<Item = syn::Result<syn::FieldValue>> + '_ {
        let Self {
            error,
            format,
            std,
            tracing,
            lifetime,
            split,
            ..
        } = self;

        self.fields().map(move |field| {
            let field = field?;
            let FieldNamed {
                ident,
                ty,
                skip,
                as_,
            } = &field;

            let split: syn::Expr = if let Some(skip) = skip {
                syn::parse_quote!(#split.by_ref().skip(#skip))
            } else {
                syn::parse_quote!(#split)
            };
            Ok(syn::parse_quote!(
                #ident: {
                    let _span = #tracing::info_span!(#std::stringify!(#ident)).entered();
                    <#as_ as #format::DecodeAs<#lifetime, #ty>>::decode_as(
                        #split.next().ok_or(#error::Error::InsufficientData)?
                    )?
                }
            ))
        })
    }
}

struct Attrs {
    split: syn::Expr,
}
impl Generator<'_> {
    fn attrs(&self) -> syn::Result<Attrs> {
        let mut split = None;
        for attr in &self.input.attrs {
            if attr.path().is_ident("format") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("split") {
                        split = Some(meta.value()?.parse()?);
                        Ok(())
                    } else {
                        Err(meta.error("unimplemented"))
                    }
                })?;
            }
        }
        Ok(Attrs {
            split: split.ok_or_else(|| syn::Error::new(self.input.span(), "missing split"))?,
        })
    }
}

struct FieldNamed<'a> {
    ident: &'a syn::Ident,
    ty: &'a syn::Type,
    skip: Option<syn::LitInt>,
    as_: syn::Type,
}
impl Generator<'_> {
    fn fields(&self) -> impl Iterator<Item = syn::Result<FieldNamed<'_>>> + '_ {
        let Self { format, .. } = self;

        self.fields.named.iter().map(move |field| {
            field.attrs.iter().try_fold(
                FieldNamed {
                    ident: field
                        .ident
                        .as_ref()
                        .ok_or_else(|| syn::Error::new(field.span(), "missing ident"))?,
                    ty: &field.ty,
                    skip: None,
                    as_: syn::parse_quote!(#format::Same),
                },
                |mut this, attr| {
                    if attr.path().is_ident("format") {
                        attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("skip") {
                                this.skip = Some(meta.value()?.parse()?);
                                Ok(())
                            } else if meta.path.is_ident("as") {
                                this.as_ = meta.value()?.parse()?;
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
