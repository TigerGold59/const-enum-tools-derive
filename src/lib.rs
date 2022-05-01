#![feature(generic_const_exprs)]
extern crate proc_macro;
extern crate const_enum_tools;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse;

#[proc_macro_derive(VariantIterable)]
pub fn derive_answer_fn(enum_item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = parse(enum_item).unwrap();

    match ast.data {
        syn::Data::Union(_) |
        syn::Data::Struct(_) => {
            panic!(
                "Iterable is only for enums and cannot be derived for structs or unions."
            );
        },
        syn::Data::Enum(enum_field_data) => {
            let variants = enum_field_data.variants;
            let generics = ast.generics.params;
            let where_clause = ast.generics.where_clause;
            let name = ast.ident;

            let mut index: usize = 0;

            let variant_index_match_arms = variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                let res = match &variant.fields {
                    syn::Fields::Named(_) => {
                        panic!("Named fields on enum");
                    },
                    syn::Fields::Unnamed(fields) => {
                        let mapped = fields.unnamed.iter().map(|_| { quote!(_) });
                        quote!(
                            #name::#variant_name(#(#mapped),*) => {
                                #index
                            }
                        )
                    },
                    syn::Fields::Unit => {
                        quote!(
                            #name::#variant_name => {
                                #index
                            }
                        )
                    },
                };
                index += 1;
                res
            });

            let variant_names = variants.iter().map(|variant| {
                let variant_name = &variant.ident.to_string();
                quote!(
                    #variant_name
                )
            });

            let variant_count = variants.len();

            quote!(
                impl <#generics> const_enum_tools::VariantCountable <#generics> for #name #where_clause {
                    const VARIANT_COUNT: usize = #variant_count;
                }

                impl <#generics> const_enum_tools::VariantIterable <#generics> for #name #where_clause {
                    #[inline]
                    fn variant_index (&self) -> usize {
                        match self {
                            #(
                                #variant_index_match_arms
                            ),*
                        }
                    }

                    const VARIANTS: [&'static str; <Self as const_enum_tools::VariantCountable>::VARIANT_COUNT] = [#(#variant_names),*];
                }
            ).into()
        },
    }

}
