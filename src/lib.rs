extern crate proc_macro2;
extern crate proc_macro;
extern crate syn;
extern crate quote;

use syn::*;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use std::str::FromStr;

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
    impl_ground(&name,&fields)
}

fn impl_ground(name: &syn::Ident, syn_fields: &syn::Fields) -> TokenStream {
    let name2 = format!("Gql{}",name);

    let gqlname = TokenStream2::from_str(&name2).unwrap();

    let mut strukt_stream = TokenStream2::default();
    let mut from_stream = TokenStream2::default();
    let mut tuple_stream = TokenStream2::default();

    if let syn::Fields::Named(FieldsNamed{named, .. }) = syn_fields {
        let fields = named.iter().map(|f| &f.ident);
        let ftypes = named.iter().map(|f| &f.ty);

        // for ft in ftypes.clone().into_iter() {
        //     println!{"{:?}",ft};
        // }

        for (field, ftype) in fields.into_iter().zip(ftypes.into_iter()) {
            strukt_stream.extend::<TokenStream2>(
                quote!{#field: }
            );
            from_stream.extend::<TokenStream2>(
                quote!{#field: }
            );
            let (strukt_extend,from_extend,tuple_extend) = match_types(field,ftype);
            strukt_stream.extend::<TokenStream2>(
                quote!{#strukt_extend,}
            );
            from_stream.extend::<TokenStream2>(
                quote!{#from_extend,}
            );
            tuple_stream.extend::<TokenStream2>(
                tuple_extend
            )
        }
    }

    let gen = quote! {
        use juniper::*;

        #tuple_stream

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
) -> (TokenStream2, TokenStream2, TokenStream2) {
    let mut strukt_stream_extend = TokenStream2::default();
    let mut from_stream_extend = TokenStream2::default();
    let mut tuple_stream_extend = TokenStream2::default();

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
                    let (gqltyp,_,_) = match &type_path.clone().path.segments.first().unwrap().arguments{
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
                    let name2 = format!("Gql{}",f);
                    let gqlname = TokenStream2::from_str(&name2).unwrap();          
                    strukt_stream_extend = quote!{#gqlname};
                    from_stream_extend = quote!{n.#field.into()};
                }                
            }      
        }
        // Tuple Type not implemented in GraphQL Object
        // Convert tuple to struct 
        // TODO!!!
        Type::Tuple(type_tuple) => {
            from_stream_extend = quote!{n.#field.into()};            
            let mut gqlfields = TokenStream2::default();
            let mut elements = TokenStream2::default();
            let mut from_tuple = TokenStream2::default();
            let mut i: usize = 0;
            for elem in type_tuple.elems.iter() {
                let (gqlfield,_,_) = match_types(field,elem);
                gqlfields.extend::<TokenStream2>(TokenStream2::from_str(&format!("t_{}: {}",i,gqlfield)).unwrap());                                
                elements.extend::<TokenStream2>(quote!{#elem});
                if Some(elem) != type_tuple.elems.last() {
                    gqlfields.extend::<TokenStream2>(quote!{,});
                    elements.extend::<TokenStream2>(quote!{,});
                }
                // let tunder = TokenStream2::from_str(&format!("t_{}:",i)).unwrap();
                // let tdot = TokenStream2::from_str(&format!("t.{}",i)).unwrap();
                from_tuple.extend::<TokenStream2>(TokenStream2::from_str(&format!("t_{}: t.{}.into(),",i,i)).unwrap());
                i = i+1;
            }
            let gqlstruct = TokenStream2::from_str(&format!("Gql{}",field.clone().unwrap().to_string())).unwrap();            
            strukt_stream_extend = quote!{#gqlstruct};
            tuple_stream_extend = quote!{
                #[derive(GraphQLObject)]
                pub struct #gqlstruct {
                    #gqlfields
                }
                impl From<(#elements)> for #gqlstruct {
                    fn from(t: (#elements)) -> #gqlstruct {
                        #gqlstruct {
                            #from_tuple
                        }
                    }
                }
            };
        }
        Type::Array(type_array) => {          
            let typ = &type_array.elem;
            let (gqltyp,_,_) = match_types(field,typ);
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
    (strukt_stream_extend,from_stream_extend,tuple_stream_extend)
}