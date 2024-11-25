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
            let blocks = fields.iter().enumerate().map(
                |(
                    i,
                    super::FieldNamed {
                        as_,
                        skip,
                        ident,
                        ty,
                    },
                )|
                 -> syn::Block {
                    let as_ = as_.as_ref().unwrap_or(&as_default);
                    let count: syn::Expr = match (i, skip) {
                        (0, Some(skip)) => syn::parse_quote!(#skip),
                        (0, None) => syn::parse_quote!(0),
                        (_, Some(skip)) => syn::parse_quote!(#skip + 1),
                        (_, None) => syn::parse_quote!(1),
                    };
                    let split = split.as_ref().map(|split| -> syn::Expr {
                        syn::parse_quote!(
                            for _ in 0..#count {
                                __fmt::Display::fmt(&#split, f)?;
                            }
                        )
                    });
                    syn::parse_quote!({
                         #split
                        <#as_ as __format::EncodeAs<#ty>>::encode_as(&self.#ident, f)?;
                    })
                },
            );
            syn::parse_quote!(
                const _: () = {
                    use crate::format as __format;
                    use ::std::fmt as __fmt;

                    impl <#(#impl_generics,)*> __format::Encode for #ident<#(#ty_generics,)*>
                    where
                    #(#where_predicates,)*
                    {
                        fn encode(&self, f: &mut __fmt::Formatter<'_>) -> __fmt::Result {
                            #(#blocks)*
                            Ok(())
                        }
                    }
                };
            )
        }
    }
}
