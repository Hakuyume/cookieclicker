use proc_macro::TokenStream;
use quote::ToTokens;
use std::iter;

#[proc_macro_derive(Decode, attributes(decode))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => {
            let mut pat = None;
            for attr in &input.attrs {
                if attr.path().is_ident("decode") {
                    if let Err(e) = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("pat") {
                            pat = Some(meta.value()?.parse::<syn::LitChar>()?);
                            Ok(())
                        } else {
                            Err(meta.error("unimplemented"))
                        }
                    }) {
                        return e.into_compile_error().into();
                    }
                }
            }

            let self_ty = {
                let ident = &input.ident;
                let generic_params = input.generics.params.iter().map(|param| match param {
                    syn::GenericParam::Lifetime(param) => param.lifetime.to_token_stream(),
                    syn::GenericParam::Type(param) => param.ident.to_token_stream(),
                    syn::GenericParam::Const(param) => param.ident.to_token_stream(),
                });
                quote::quote!(#ident<#(#generic_params,)*>)
            };
            let generics = iter::once(quote::quote!('decode)).chain(
                input
                    .generics
                    .params
                    .iter()
                    .cloned()
                    .map(|param| match param {
                        syn::GenericParam::Lifetime(param) => param.lifetime.into_token_stream(),
                        syn::GenericParam::Type(mut param) => {
                            param.eq_token = None;
                            param.default = None;
                            param.into_token_stream()
                        }
                        syn::GenericParam::Const(mut param) => {
                            param.eq_token = None;
                            param.default = None;
                            param.into_token_stream()
                        }
                    }),
            );
            let where_predicates = fields
                .named
                .iter()
                .map(|field| {
                    let ty = &field.ty;
                    if pat.is_some() {
                        quote::quote!(#ty: crate::decode::Decode<&'decode str>)
                    } else {
                        quote::quote!(#ty: crate::decode::Decode<char>)
                    }
                })
                .chain(
                    input
                        .generics
                        .where_clause
                        .iter()
                        .flat_map(|where_clause| &where_clause.predicates)
                        .map(|predicate| predicate.into_token_stream()),
                );
            let segments = if let Some(pat) = &pat {
                quote::quote!(value.split(#pat))
            } else {
                quote::quote!(value.chars())
            };
            let field_values = fields.named.iter().map(|field| {
                let mut skip = None;
                for attr in &field.attrs {
                    if attr.path().is_ident("decode") {
                        if let Err(e) = attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("skip") {
                                skip = Some(meta.value()?.parse::<syn::LitInt>()?);
                                Ok(())
                            } else {
                                Err(meta.error("unimplemented"))
                            }
                        }) {
                            return e.into_compile_error();
                        }
                    }
                }

                let ident = field.ident.as_ref().unwrap();
                let segments = if let Some(skip) = &skip {
                    quote::quote!(segments.by_ref().skip(#skip))
                } else {
                    quote::quote!(segments)
                };

                quote::quote!(
                    #ident: crate::decode::Decode::decode(
                        #segments.next().ok_or(crate::error::Error::InsufficientData)?
                    )?
                )
            });

            quote::quote!(
                impl<#(#generics,)*> crate::decode::Decode<&'decode str> for #self_ty
                where
                #(#where_predicates,)*
                {
                    fn decode(value: &'decode str) -> Result<Self, crate::error::Error> {
                        let mut segments = #segments;
                        Ok(Self { #(#field_values,)* })
                    }
                }
            )
            .into()
        }
        _ => quote::quote!(::core::compile_error!("unimplemented")).into(),
    }
}
