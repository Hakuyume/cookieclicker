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
                    syn::parse_quote!(#as_: __format::DecodeAs<'__decode, #ty>)
                },
            ));
            let split: syn::Expr = if let Some(split) = split {
                syn::parse_quote!(value.split(#split))
            } else {
                syn::parse_quote!(__format::chars(value))
            };
            let field_values = fields.iter().map(
                |super::FieldNamed { as_, ident, ty }| -> syn::FieldValue {
                    let as_ = as_.as_ref().unwrap_or(&as_default);
                    syn::parse_quote!(
                        #ident: {
                            let _span = __tracing::info_span!(__std::stringify!(#ident)).entered();
                            <#as_ as __format::DecodeAs<'__decode, #ty>>::decode_as(
                                split.next().ok_or(__error::Error::InsufficientData)?
                            )?
                        }
                    )
                },
            );
            syn::parse_quote!(
                const _: () = {
                    use crate::error as __error;
                    use crate::format as __format;
                    use ::std as __std;
                    use ::tracing as __tracing;

                    impl <'__decode, #(#impl_generics,)*> __format::Decode<'__decode> for #ident<#(#ty_generics,)*>
                    where
                    #(#where_predicates,)*
                    {
                        #[__tracing::instrument(err)]
                        fn decode(value: &'__decode str) -> Result<Self, __error::Error> {
                            let mut split = #split;
                            Ok(Self {#(#field_values,)*})
                        }
                    }
                };
            )
        }
    }
}
