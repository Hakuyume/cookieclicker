use std::iter;

pub(super) fn derive(input: &super::Input) -> syn::Item {
    let impl_generics = input.impl_generics();
    let ty_generics = input.ty_generics();
    let where_predicates = input.where_predicates();

    let as_default = syn::parse_quote!(__format::Same);

    match input {
        super::Input::StructNamed {
            split,
            ident,
            fields,
            ..
        } => {
            let where_predicates = where_predicates.chain(fields.iter().map(
                |super::FieldNamed { as_, ty, .. }| {
                    let as_ = as_.as_ref().unwrap_or(&as_default);
                    syn::parse_quote!(#as_: __format::EncodeAs<#ty>)
                },
            ));
            let exprs =
                fields
                    .iter()
                    .enumerate()
                    .flat_map(|(i, super::FieldNamed { as_, ident, ty })| {
                        let as_ = as_.as_ref().unwrap_or(&as_default);
                        split
                            .iter()
                            .filter(move |_| i > 0)
                            .map(|split| -> syn::Expr {
                                syn::parse_quote!(__fmt::Display::fmt(&#split, f))
                            })
                            .chain(iter::once(syn::parse_quote!(
                                <#as_ as __format::EncodeAs<#ty>>::encode_as(&self.#ident, f)
                            )))
                    });
            syn::parse_quote!(
                const _: () = {
                    use crate::format as __format;
                    use ::std::fmt as __fmt;

                    impl <#(#impl_generics,)*> __format::Encode for #ident<#(#ty_generics,)*>
                    where
                    #(#where_predicates,)*
                    {
                        fn encode(&self, f: &mut __fmt::Formatter<'_>) -> __fmt::Result {
                            #(#exprs?;)*
                            Ok(())
                        }
                    }
                };
            )
        }
    }
}
