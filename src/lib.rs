extern crate proc_macro2;
extern crate proc_macro;
extern crate syn;
extern crate quote;

use syn::*;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use std::str::FromStr;
use std::any::TypeId;

#[proc_macro_derive(Ground)]
pub fn ground_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let output = input.clone();
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let strukt = &ast.data;
    let fields = match strukt {
        syn::Data::Struct(d) => &d.fields,
        syn::Data::Enum(e) => {
            let name2 = format!("Gql{}",name);
            let gqlname = TokenStream2::from_str(&name2).unwrap();            
            let mut out_extend = quote!();
            let mut from_extend = quote!();
            // if let syn::Fields::Named(FieldsNamed{named,..}) = &e.variants.first().unwrap().fields {
            //     for v in named.iter().map(|f| &f.ident) {
            for variant in e.variants.iter() {
                let v = &variant.ident;
                out_extend.extend::<TokenStream2>(quote!{
                    #v,
                });
                from_extend.extend::<TokenStream2>(quote!{
                    #name::#v => #gqlname::#v,
                });
            }
            let out = quote!{
                #[derive(GraphQLEnum)]
                pub enum #gqlname {
                    #out_extend
                }
                impl From<#name> for #gqlname {
                    fn from(e: #name) -> #gqlname {
                        match e {
                            #from_extend
                        }
                    }
                }
            };
            println!("{}",out);
            return out.into()

        }
        _ => return output,
    };

    // Build the trait implementation
    impl_ground(&name,&fields,output)
}

fn impl_ground(name: &syn::Ident, syn_fields: &syn::Fields, tok: TokenStream) -> TokenStream {
    // let name = &ast.ident;    
    // let data = &ast.data;    
    let input = tok.to_string();
    let name2 = format!("Gql{}",name);

    let gqlname = TokenStream2::from_str(&name2).unwrap();

    let mut strukt_stream = TokenStream2::default();
    let mut from_stream = TokenStream2::default();

    if let syn::Fields::Named(FieldsNamed{named, .. }) = syn_fields {
        let fields = named.iter().map(|f| &f.ident);
        let ftypes = named.iter().map(|f| &f.ty);

        for ft in ftypes.clone().into_iter() {
            println!{"{:?}",ft};
        }

        for (field, ftype) in fields.into_iter().zip(ftypes.into_iter()) {
            strukt_stream.extend::<TokenStream2>(
                quote!{#field: }
            );
            from_stream.extend::<TokenStream2>(
                quote!{#field: }
            );
            let (strukt_extend,from_extend) = match_types(field,ftype);
            strukt_stream.extend::<TokenStream2>(
                quote!{#strukt_extend,}
            );
            from_stream.extend::<TokenStream2>(
                quote!{#from_extend,}
            );
        }
    }

    let gen = quote! {
        use juniper::*;

        #[derive(GraphQLObject)]
        pub struct #gqlname {
            #strukt_stream
        }
        
        impl From<#name> for #gqlname {
            fn from(n: #name) -> #gqlname {
                #gqlname {
                    #from_stream
                }
            }
        }
    };

    println!("{}",gen);
    gen.into()
}

fn match_types(
    field: &Option<syn::Ident>,
    ftype: &syn::Type,
) -> (TokenStream2, TokenStream2) {
    let mut strukt_stream_extend = TokenStream2::default();
    let mut from_stream_extend = TokenStream2::default();

    match ftype {
        Type::Path(type_path) => {
            match type_path.clone().path.segments.first().unwrap().ident.to_string().as_str() {
                "u8" => {
                    strukt_stream_extend = quote!{i32};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "u16" => {
                    strukt_stream_extend = quote!{i32};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "u32" => {
                    strukt_stream_extend = quote!{i32};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "u64" => {
                    strukt_stream_extend = quote!{f64};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "i8" => {
                    strukt_stream_extend = quote!{i32};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "i16" => {
                    strukt_stream_extend = quote!{i32};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "i32" => {
                    strukt_stream_extend = quote!{i32};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "i64" => {
                    strukt_stream_extend = quote!{f64};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "usize" => {
                    strukt_stream_extend = quote!{f64};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "isize" => {
                    strukt_stream_extend = quote!{f64};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "f32" => {
                    strukt_stream_extend = quote!{f64};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "f64" => {
                    strukt_stream_extend = quote!{f64};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "bool" => {
                    strukt_stream_extend = quote!{bool};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "String" => {
                    strukt_stream_extend = quote!{String};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "&str" => {
                    strukt_stream_extend = quote!{String};
                    from_stream_extend = quote!{n.#field.to_string()};
                }
                "Vec" => {                   
                    let (gqltyp,_) = match &type_path.clone().path.segments.first().unwrap().arguments{
                        PathArguments::AngleBracketed(a) => match a.args.first().unwrap() {
                            GenericArgument::Type(f) => match_types(field,f),     
                            _ => todo!(),                       
                        }                        
                        _ => todo!(),
                    };
                    from_stream_extend = quote!{
                        {
                            let mut v: Vec<#gqltyp> = Vec::new();
                            for i in n.#field.iter()  {
                                v.push(<#gqltyp>::from(*i));
                            }
                            v
                        }
                    };
                    strukt_stream_extend = quote!{Vec<#gqltyp>};
                }
                _ => {        
                    let f = type_path.clone().into_token_stream();
                    println!("{:?}",f);
                    let name2 = format!("Gql{}",f);
                    let gqlname = TokenStream2::from_str(&name2).unwrap();          
                    strukt_stream_extend = quote!{#gqlname};
                    from_stream_extend = quote!{n.#field.into()};
                }                
            }
            // match type_path.clone().into_token_stream().to_string().as_str() {
            //     "u8" => {
            //         strukt_stream_extend = quote!{i32};
            //         from_stream_extend = quote!{n.#field.into()};
            //     }
            //     
            //     x => {
            //         strukt_stream_extend = quote!{#x};
            //         from_stream_extend = quote!{n.#field.into()};
            //     }
            // }            
        }
        /// Tuple Type not implemented in GraphQL Object
        /// Convert tuple to struct 
        /// TODO!!!
        // Type::Tuple(type_tuple) => {
        //     from_stream_extend = quote!{n.#field.into()};
        //     let mut gqltyp = TokenStream2::default();
        //     for elem in type_tuple.elems.iter() {
        //         gqltyp.extend::<TokenStream2>(
        //             match_types(field,elem).0
        //         );                
        //         gqltyp.extend::<TokenStream2>(quote!{,})
        //     }
        //     strukt_stream_extend = quote!((#gqltyp));
        // }
        Type::Array(type_array) => {          
            let typ = &type_array.elem;
            let (gqltyp,_) = match_types(field,typ);
            strukt_stream_extend = quote!{Vec<#gqltyp>};
            if let syn::Expr::Lit(expr_lit) = &type_array.len {
                from_stream_extend = quote!{
                    {
                        let mut v: Vec<#gqltyp> = Vec::new();
                        for i in 0..#expr_lit {
                            v.push(n.#field[i].into());
                        }
                        v
                    }
                };
            }
        }
        _ => {}                    
    };
    (strukt_stream_extend,from_stream_extend)
}