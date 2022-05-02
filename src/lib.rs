#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
extern crate proc_macro;
extern crate const_enum_tools;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(VariantIterable)]
pub fn derive_answer_fn(enum_item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = parse_macro_input!(enum_item as DeriveInput);

    match ast.data {
        syn::Data::Union(union_data) => {
            let err = syn::Error::new_spanned(union_data.union_token, "Unexpected union declaration: VariantIterable can only be derived for enums.");
            err.into_compile_error().into()
        },
        syn::Data::Struct(struct_data) => {
            let err = syn::Error::new_spanned(struct_data.struct_token, "Unexpected union declaration: VariantIterable can only be derived for enums.");
            err.into_compile_error().into()
        },
        syn::Data::Enum(enum_field_data) => {
            let variants = enum_field_data.variants;
            let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
            let name = ast.ident;
            let variant_count = variants.len();

            let mut variant_index_match_arms = Vec::new();
            let mut variant_names = Vec::new();
            let mut all_unit_no_discriminant = true;

            for (index, variant) in variants.iter().enumerate() {
                let variant_name = &variant.ident;

                variant_index_match_arms.push(
                    match &variant.fields {
                        syn::Fields::Named(fields) => {
                            all_unit_no_discriminant = false;
                            let mapped = fields.named.iter().map(|_| { quote!(_) });
                            quote!(
                                #name::#variant_name(#(#mapped),*) => {
                                    #index
                                }
                            )
                        },
                        syn::Fields::Unnamed(fields) => {
                            all_unit_no_discriminant = false;
                            let mapped = fields.unnamed.iter().map(|_| { quote!(_) });
                            quote!(
                                #name::#variant_name(#(#mapped),*) => {
                                    #index
                                }
                            )
                        },
                        syn::Fields::Unit => {
                            // If there is an explicit discriminant, we might not be able to perform the bitwise copy
                            // optimization.
                            if let Some(discriminant) = &variant.discriminant {
                                match discriminant.1.clone() {
                                    // If the discriminant expression is a literal, we can check if it is equal to the default value.
                                    syn::Expr::Lit(lit) => {
                                        match lit.lit {
                                            syn::Lit::Int(int_lit) => {
                                                // If the first part of the literal before the type is the same as what it would be
                                                // because of the position in the enum, we're good. Otherwise, no optimization.
                                                if int_lit.base10_digits() != index.to_string().as_str() {
                                                    all_unit_no_discriminant = false;
                                                }
                                            },
                                            _ => {
                                                all_unit_no_discriminant = false;
                                            }
                                        }
                                    },
                                    // Otherwise, since we cannot evaluate arbitrary const expressions, we will not be able to optimize.
                                    // This involves using the long match arms list.
                                    _ => {
                                        all_unit_no_discriminant = false;
                                    },
                                }
                            }
                            quote!(
                                #name::#variant_name => {
                                    #index
                                }
                            )
                        },
                    }
                );

                variant_names.push({
                    let variant_name_string = variant_name.to_string();
                    quote!(
                        #variant_name_string
                    )
                });

            }

            let variant_countable_impl = quote!(
                impl #impl_generics const_enum_tools::VariantCountable for #name #ty_generics #where_clause {
                    const VARIANT_COUNT: usize = #variant_count;
                }
            );

            // If there are no explicit discriminants
            // This enum will be represented as a number type. Cast the reference
            // to a raw pointer and read the bits from it (allows this optimization to be performed even when self =/= Copy).
            // This is effectively a clone. Then cast to usize for index.
            // I would love a better way of doing this that doesn't require an unsafe block. Alas, I can't think of any.
            let variant_index_body = if all_unit_no_discriminant {
                quote!(
                    unsafe {
                        (self as *const Self).read() as usize
                    }
                )
            }
            else {
                quote!(
                    match self {
                        #(
                            #variant_index_match_arms
                        ),*
                    }
                )
            };

            quote!(
                #variant_countable_impl

                impl #impl_generics const_enum_tools::VariantIterable for #name #ty_generics #where_clause {
                    #[inline]
                    fn variant_index (&self) -> usize {
                        #variant_index_body
                    }

                    const VARIANTS: [&'static str; #variant_count] = [#(#variant_names),*];
                }
            ).into()
        }
    }

}
