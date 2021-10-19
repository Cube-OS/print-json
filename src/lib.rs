extern crate proc_macro;
extern crate syn;
extern crate quote;
use proc_macro::TokenStream;
use quote::*;

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
            fn deserialize_sized(array: &[u8]) -> Result<Self> where Self:Sized {
                if array.len() != size_of::<#name>() {
                    Err(CubeOSError::WrongNoArgs)
                } else {
                    Ok(bincode::deserialize(array).unwrap())
                }   
            }
        }
    };
    gen.into()
}