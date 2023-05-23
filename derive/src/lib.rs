//! Provides macro for deriving static EthABI decode implementations
//!
//! Intended for high speed decoding, not feature completeness
//! Trades binary size for performance vs. ethabi
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, spanned::Spanned, Attribute, Data, DeriveInput, Fields, Meta, NestedMeta};

#[proc_macro_derive(DecodeStatic, attributes(ethabi))]
pub fn decode_static_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = match syn::parse(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error().into(),
    };

    let name = &input.ident;
    let steps = decode_steps(input.data);

    // TODO: do this with one quote...
    // support 1 lifetime and 1 generic only
    let lifetime = input.generics.lifetimes().next();
    let generic = input.generics.type_params().next();

    match (lifetime, generic) {
        (Some(lifetime), Some(generic)) => {
            quote! {
                impl<#lifetime, #generic> DecodeStatic<#lifetime> for #name<#lifetime, #generic>
                where
                    #generic: DecodeStatic<#lifetime>
                {
                    fn decode_static(buf: &#lifetime [u8], offset: usize) -> Result<Self, ()> {
                        #steps
                    }
                }
            }
        }
        (Some(lifetime), None) => {
            quote! {
                impl<#lifetime> DecodeStatic<#lifetime> for #name<#lifetime> {
                    fn decode_static(buf: &#lifetime [u8], offset: usize) -> Result<Self, ()> {
                        #steps
                    }
                }
            }
        }
        (None, Some(generic)) => {
            quote! {
                impl<'a, #generic> DecodeStatic<'a> for #name<#generic>
                where
                    #generic: DecodeStatic<'a>
                {
                    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
                        #steps
                    }
                }
            }
        }
        _ => {
            quote! {
                impl<'a> DecodeStatic<'a> for #name {
                    fn decode_static(buf: &'a [u8], offset: usize) -> Result<Self, ()> {
                        #steps
                    }
                }
            }
        }
    }
    .into()
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
                    let offset = 32_usize * idx;
                    let type_string = f_type.to_token_stream().to_string().replace(" ", "");

                    let is_list = type_string.starts_with("Vec");
                    let field_is_dynamic: bool = is_list || type_string.starts_with("BytesZcp");

                    if should_skip(&f.attrs) {
                        tail_stmts.push(quote! {
                            #f_name: Default::default(),
                        });
                        continue;
                    }

                    if !field_is_dynamic {
                        head_stmts.push(quote! {
                            let #f_name = <#f_type>::decode_static(buf, #offset)?;
                        });
                        tail_stmts.push(quote! {
                            #f_name,
                        });
                        continue;
                    }

                    // if dynamic we read the head then decode tail after
                    head_stmts.push(
                        quote! {
                            let #f_name = ((unsafe { *buf.get_unchecked(#offset + 30) } as usize) << 8) + (unsafe { *buf.get_unchecked(#offset + 31) } as usize);
                        }
                    );

                    if is_list {
                        let mut ts = f_type.clone().into_token_stream().into_iter();
                        let dynamic_inner =
                            if let Some(proc_macro2::TokenTree::Ident(list_type)) = ts.nth(2) {
                                if list_type == "Vec" {
                                    unimplemented!("nested arrays unsupported");
                                }
                                list_type.to_string() == "BytesZcp"
                            } else {
                                false
                            };

                        tail_stmts.push(quote! {
                            #f_name: <_ethabi_static::Array<_, #dynamic_inner>>::decode_static(buf, #f_name)?.0,
                        });
                    } else {
                        tail_stmts.push(quote! {
                            #f_name: <#f_type>::decode_static(buf, #f_name)?,
                        });
                    }
                }

                quote! {
                    extern crate ethabi_static as _ethabi_static;
                    #(#head_stmts)*
                    Ok(Self {
                        #(#tail_stmts)*
                    })
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

/// Look for a `#[ethabi(skip)]` in the given attributes.
fn should_skip(attrs: &[Attribute]) -> bool {
    find_meta_item(attrs.iter(), |meta| {
        if let NestedMeta::Meta(Meta::Path(ref path)) = meta {
            if path.is_ident("skip") {
                return Some(path.span());
            }
        }

        None
    })
    .is_some()
}

fn find_meta_item<'a, F, R, I, M>(mut itr: I, mut pred: F) -> Option<R>
where
    F: FnMut(M) -> Option<R> + Clone,
    I: Iterator<Item = &'a Attribute>,
    M: Parse,
{
    itr.find_map(|attr| {
        attr.path
            .is_ident("ethabi")
            .then(|| pred(attr.parse_args().ok()?))
            .flatten()
    })
}
