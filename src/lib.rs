extern crate proc_macro;
extern crate syn;
extern crate quote;
use proc_macro::TokenStream;
use quote::*;
use std::error::Error;
use cubeos_error::*;

#[proc_macro_derive(DeserializeSized)]
pub fn deserialize_sized_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_deserialize_sized(&ast)
}

fn impl_deserialize_sized(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl DeserializeSized for #name where {
            fn deserialize_sized(array: &[u8], sized: std::option::Option<usize>) -> DeResult<Self> where Self:Sized {
                match sized {
                    Some(l) => {
                        if array.len() != l {
                            Err(DError::Deserialize)
                        } else {
                            Ok(bincode::deserialize(array).unwrap())
                        }                        
                    }
                    None => Ok(bincode::deserialize(array).unwrap())
                }
            }
        }
    };
    gen.into()
}