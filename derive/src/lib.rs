//! Provides macro for deriving static EthABI decode implementations
//!
//! Intended for high speed decoding, not feature completeness
//! Trades binary size for performance vs. ethabi
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Fields};

#[proc_macro_derive(DecodeStatic)]
pub fn decode_static_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = match syn::parse(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error().into(),
    };

    let name = &input.ident;
    let steps = decode_steps(input.data);
    // quick-n-dirty lifetime no lifetime (will fail on generic structs)
    if input.generics.lt_token.is_some() {
        quote! {
            impl<'a> DecodeStatic<'a> for #name<'a> {
                fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
                    #steps
                }
            }
        }
        .into()
    } else {
        quote! {
            impl<'a> DecodeStatic<'a> for #name {
                fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
                    #steps
                }
            }
        }
        .into()
    }
}

fn decode_steps(data: Data) -> TokenStream {
    match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields_named) => {
                let len = fields_named.named.len();
                let mut head_stmts = Vec::<TokenStream>::with_capacity(len);
                let mut tail_stmts = Vec::<TokenStream>::with_capacity(len);

                for (idx, f) in fields_named.named.iter().enumerate() {
                    let f_name = f.ident.clone().unwrap();
                    let f_type = &f.ty;
                    let offset = idx * 32_usize;
                    let type_string = f_type.to_token_stream().to_string();
                    // could add more types here..
                    let is_dynamic =
                        type_string.starts_with("Bytes") || type_string.starts_with("Vec"); // Vec equivalent to solidity Array
                                                                                            // always read head values then tail values for better locality
                    if !is_dynamic {
                        head_stmts.push(
                            quote! {
                                let #f_name = <#f_type>::decode_static(buf, #offset)?;
                            }
                            .into(),
                        );
                        tail_stmts.push(
                            quote! {
                                #f_name,
                            }
                            .into(),
                        );
                    } else {
                        // if dynamic we read the head then decode tail after
                        head_stmts.push(
                            quote! {
                                let #f_name = ((unsafe { *buf.get_unchecked(#offset + 30) } as usize) << 8) + (unsafe { *buf.get_unchecked(#offset + 31) } as usize);
                            }
                            .into(),
                        );
                        tail_stmts.push(
                            quote! {
                                #f_name: <#f_type>::decode_static(buf, #f_name)?,
                            }
                            .into(),
                        );
                    }
                }

                quote! {
                    #(#head_stmts)*
                    Ok(Self {
                        #(#tail_stmts)*
                    })
                }
                .into()
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}
