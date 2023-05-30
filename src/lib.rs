use syn::*;
use syn::spanned::Spanned;
use syn::punctuated::Punctuated;
use syn::parse::Parser;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use std::str::FromStr;
use std::marker::PhantomData;
use std::cell::Cell;
use std::rc::Rc;
use std::fmt::Display;
use cargo_metadata::*;
use std::fs;
use std::io::Write;
use serde_json::{json, to_string_pretty};
use std::io::Read;

#[proc_macro]
pub fn print_json(input: TokenStream) -> TokenStream {

    // let parser = Punctuated::<TokenStream2, Token![;]>::parse_terminated;
    // let mut args = parser.parse2(input.into()).unwrap();

    // let mut vec = args.into_iter().collect::<Vec<_>>();
    // let 
    let input = input.to_string();
    let input = input.replace(",,", ",");
    let mut vec_str: Vec<String> = input.split(";").map(|s| s.to_string()).collect();
    vec_str.pop();
    for v in &vec_str {
        println!("{}", v);
    }

    let mut enumstruct = String::new();
    let mut commands = String::new();
    for mut v in vec_str {  
        println!("v: {}", v);      
        let mut input = TokenStream::from_str(&v).unwrap();
        let parser = Punctuated::<TypePath, Token![,]>::parse_terminated;
        let mut args = parser.parse(input).unwrap();

        let mut vec = args.into_iter().collect::<Vec<_>>();      
        
        let mut command = "# struct ".to_string();
        println!("{:?}", command);
        
        command.push_str(&vec.remove(0).path.segments.first().unwrap().ident.to_string());
        
        let (name, typ): (Vec<_>,Vec<_>) = vec.into_iter().enumerate().partition(|(i, _)| i % 2 == 0);
        
        let mut json = String::new();        
        for ((_,n),(_,t)) in name.into_iter().zip(typ.into_iter()) {
            json.push_str(&format!("\t{}: ", n.path.segments.first().unwrap().ident.to_string()));
            let (s,t) = arguments(t);
            json.push_str(&format!("{},\n", s));
            if t.is_some() {
                enumstruct.push_str(&format!("{}",t.unwrap()));
            }        
        }
        
        if !json.is_empty() {
            command.push_str(&format!(" {{\n{}}}\n", json));
        } else {
            command.push_str(" {}\n");
        }

        commands.push_str(&format!("{}\n", command));
    }
    
    commands.push_str(&format!("{}\n", remove_duplicates(&enumstruct)));
    println!("{}", commands);
    // println!("{}", enumstruct);

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("commands.json")
        .expect("Failed to open file");

    write!(file, "{}", commands).expect("Failed to write to file");
    // write!(file, "{}", enumstruct).expect("Failed to write to file");

    TokenStream::new()
}

fn remove_duplicates(s: &str) -> String {
    let mut items: Vec<syn::Item> = syn::parse_file(s).unwrap().items;

    let mut unique_items: Vec<syn::Item> = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();

    for item in items {
        match &item {
            Item::Struct(item_struct) => {
                if seen.insert(item_struct.ident.to_string()) {
                    unique_items.push(item);
                }
            }
            Item::Enum(item_enum) => {
                println!("{:?}",item_enum.ident.to_string());
                if seen.insert(item_enum.ident.to_string()) {
                    unique_items.push(item);
                }
            }
            _ => {}
        }
    }

    println!("{:?}",seen);

    let mut output = String::new();
    for item in &unique_items {
        output.push_str(&format!("{}\n", item.to_token_stream().to_string()));
    }
    output
}

// #[proc_macro]
fn arguments(path: TypePath) -> (String,Option<String>) {
    let depth: usize = 0;
    let mut enumstruct = String::new();
    let (s,t) = match_name(&path,depth,enumstruct);
    (s.unwrap_or_else(|| "".to_string()),t)
}

fn match_name(type_path: &TypePath, depth: usize, mut enumstruct: String) -> (Option<String>,Option<String>) {
    let depth = depth + 1;
    println!("{}", type_path.clone().path.segments.first().unwrap().ident.to_string());
    match type_path.clone().path.segments.first().unwrap().ident.to_string().as_str() {
        "u8" => {
            (Some(format!("u8")),None)
        }
        "u16" => {
            (Some(format!("u16")),None)
        }
        "u32" => {
            (Some(format!("u32")),None)
        }
        "u64" => {
            (Some(format!("u64")),None)
        }
        "i8" => {
            (Some(format!("i8")),None)
        }
        "i16" => {
            (Some(format!("i16")),None)
        }
        "i32" => {
            (Some(format!("i32")),None)
        }
        "i64" => {
            (Some(format!("i64")),None)
        }
        "f32" => {
            (Some(format!("f32")),None)
        }
        "f64" => {
            (Some(format!("f64")),None)
        }
        "String" => {
            (Some(format!("String")),None)
        }
        "bool" => {
            (Some(format!("bool")),None)
        }
        "Vec" => {
            let (arg,typ) = match &type_path.clone().path.segments.first().unwrap().arguments{
                PathArguments::AngleBracketed(a) => match a.args.first().unwrap() {
                    GenericArgument::Type(t) => match t {
                        Type::Path(f) => match_name(f,depth,enumstruct.clone()),
                        _ => todo!(),
                    },
                    _ => todo!(),                       
                }                        
                _ => todo!(),
            };
            enumstruct.push_str(&typ.unwrap_or_else(|| "".to_string()));
            (Some(format!("Vec<{}>",arg.unwrap())),Some(enumstruct))
        }
        "Option" => {
            let (arg,typ) = match &type_path.clone().path.segments.first().unwrap().arguments{
                PathArguments::AngleBracketed(a) => match a.args.first().unwrap() {
                    GenericArgument::Type(t) => match t {
                        Type::Path(f) => match_name(f,depth,enumstruct.clone()),
                        _ => todo!(),
                    },
                    _ => todo!(),                       
                }                        
                _ => todo!(),
            };
            enumstruct.push_str(&typ.unwrap_or_else(|| "".to_string()));
            (Some(format!("Option<{}>",arg.unwrap())),Some(enumstruct))
        }
        id => {
            let id: TokenStream2 = id.parse().unwrap();
            handle_ident(id,depth,enumstruct.clone())
        }
    }   
}

fn recursive_find_path(use_path: &UsePath, ident: &Ident) -> Option<String> {
    // println!("{} {}",use_path.ident,ident);
    if use_path.ident == *ident {
        // println!("found path: {}",use_path.to_token_stream().to_string());
        Some(use_path.to_token_stream().to_string())
    } else {
        if let use_tree = use_path.tree.as_ref() {
            // println!("tree: {}",use_tree.to_token_stream().to_string());
            // println!("{:?}",use_tree);
            match use_tree {
                UseTree::Path(use_path) => recursive_find_path(use_path, ident),
                UseTree::Name(use_name) => {
                    if use_name.ident == *ident {
                        // println!("found path: {}",use_name.to_token_stream().to_string());
                        Some(use_name.to_token_stream().to_string())
                    } else {
                        // println!("not found");
                        None
                    }
                }
                _ => None,
            }
        } else {
            // println!("no tree");
            None
        }        
    }
}

fn find_path(file_ast: syn::File, ident: &Ident) -> Option<String> {
    for item in file_ast.items.clone() {
        match item {
            Item::Use(item_use) => {
                if let UseTree::Path(use_path) = item_use.tree {
                    match recursive_find_path(&use_path, ident) {
                        Some(_) => return Some(use_path.to_token_stream().to_string()),
                        None => (),
                    }
                }
            },
            _ => (),
        }
    }
    None
}

fn find_struct_or_enum_definition(ident: &Ident) -> Option<Item> {
    // Get the file path of the current module - fix this to /src/service.rs for now
    let module_path = std::path::Path::new(&std::env::current_dir().unwrap()).join("src").join("service.rs");
    // println!("{:?}",module_path);
    let file_content = std::fs::read_to_string(module_path).unwrap();    
    // Parse the file into a Syn abstract syntax tree (AST)
    let file_ast = syn::parse_file(&file_content).unwrap();
    // println!("file_ast: {:?}",file_ast);

    match find_path(file_ast.clone(), ident) {
        Some(path) => {
            // println!("path: {}",path);
            if path.contains("crate ::") {
                let path = path.split("::").collect::<Vec<&str>>();
                let krate = path[path.len()-2];                
                // println!("{}",krate);
                let module_path = std::path::Path::new(&std::env::current_dir().unwrap()).join("src").join((String::from(krate)+".rs").replace(" ",""));
                // println!("{:?}",module_path);
                let file_content = std::fs::read_to_string(module_path).unwrap();
                let file_ast = syn::parse_file(&file_content).unwrap();
                // println!("here");
                
                for item in file_ast.items {
                    match item {
                        Item::Struct(item_struct) => {
                            if item_struct.ident == *ident {
                                return Some(Item::Struct(item_struct));
                            }
                        },
                        Item::Enum(item_enum) => {
                            if item_enum.ident == *ident {
                                return Some(Item::Enum(item_enum));
                            }
                        },
                        _ => (),
                    }
                }
                None
            } else if path.contains(ident.to_string().as_str()) {
                let package = Some(path.split("::").collect::<Vec<&str>>()[0].replace("_","-").trim_end().to_string());
                read_from_git_dependency(package,ident)
            } else {
                read_from_git_dependency(None,ident)
            }
        },
        None => read_from_git_dependency(None,ident),
    }
}

fn find_in_git(package: &Package, ident: &syn::Ident) -> Option<Item> {
    // Get path to git dependency crate
    let directory = package.manifest_path.parent().unwrap().as_std_path();

    // println!("searching in: {:?}",directory);
    match search_files(&directory, ident) {
        Ok(item) => Some(item),
        Err(_) => None,
    }
}

fn search_files(directory: &std::path::Path, ident: &syn::Ident) -> std::result::Result<Item,Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        // Skip all dependencies from crates.io in the `/root/.cargo/registry` directory
        if path.starts_with("/root/.cargo/registry") {
            continue;
        } else if path.is_dir() {
            //Recurse into subdirectory
            if let Ok(item) = search_files(&path, ident) {
                return Ok(item);
            }
        } else if path.extension().map(|ext| ext == "rs").unwrap_or(false) {
            // println!("searching in: {:?}",path);
            // Parse source files
            let file_content = std::fs::read_to_string(path.clone())?;
            let file_ast = syn::parse_file(&file_content)?;

            for item in file_ast.items {
                match item {
                    Item::Struct(item_struct) => {
                        if item_struct.ident == *ident {
                            // println!("found in: {:?}",path);
                            return Ok(Item::Struct(item_struct));
                        }
                    },
                    Item::Enum(item_enum) => {
                        if item_enum.ident == *ident {
                            // println!("found in: {:?}",path);
                            return Ok(Item::Enum(item_enum));
                        }
                    },
                    _ => (),
                }
            }
        }
    }
    Err("not found".into())
}

fn read_from_git_dependency(package_name: Option<String>, ident: &syn::Ident) -> Option<Item> {
    // Get path to Cargo.toml
    let manifest_path = std::env::current_dir().unwrap().join("Cargo.toml");
    // Load Cargo project metadata
    let metadata: Metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(manifest_path)
        .exec()
        .unwrap();

    // Iterate over all dependencies
    for package in metadata.packages {        
        if package_name.is_some() && package.name != package_name.clone().unwrap() {
            continue;
        }
        // println!("package: {:?}",package.name);
        match find_in_git(&package, ident) {
            Some(item) => return Some(item),
            None => (),
        }
        // // if package.source.is_some() && package.source.as_ref().unwrap().is_git() && package.name == ident.to_string() {
        // if package.source.is_some() {            
            
        // }
    }
    None
}

fn handle_ident(ident: TokenStream2,depth: usize, enumstruct: String) -> (Option<String>,Option<String>) {    
    find_struct_or_enum_definition(&parse2::<Ident>(ident.clone()).unwrap()).map(|item| {
        match item {
            Item::Struct(item_struct) => {
                handle_struct(item_struct,depth,enumstruct)                
            },
            Item::Enum(item_enum) => {
                handle_enum(item_enum,depth,enumstruct)
            },
            _ => {
                (None,None)
            }
        }
    }).unwrap()
}

fn handle_struct_fields(item_struct: ItemStruct, depth: usize, mut enumstruct: String) -> (String,Option<String>) {
    let mut field_streams = String::new();
    for field in item_struct.fields {
        let field_name = field.clone().ident.unwrap();
        let (field_type,field_enumstruct) = match field.clone().ty {
            syn::Type::Path(path) => {
                match_name(&path,depth-1,enumstruct.clone())
            },
            _ => (Some("".to_string()),Some("".to_string())),
        };
        field_streams.push_str(format!{"\t{}: {},\n",field_name,field_type.unwrap()}.as_str());
        enumstruct.push_str(&format!("{}",field_enumstruct.unwrap_or("".to_string())));
    }    
    (field_streams,Some(enumstruct))
}

fn handle_struct(item_struct: ItemStruct, depth: usize, mut enumstruct: String) -> (Option<String>,Option<String>) {
    let struct_name = item_struct.clone().ident;
    let (struct_fields, fields_enumstruct) = handle_struct_fields(item_struct.clone(),depth+1,enumstruct.clone());
    enumstruct.push_str(&format!("struct {} {{\n{}\n}}\n",
        struct_name,
        struct_fields,
    ));
    enumstruct.push_str(&format!("{}",fields_enumstruct.unwrap_or("".to_string())));
    (Some(format!("{}",struct_name)),Some(enumstruct))
    // Some(format!("{} {{\n{}{}}}",struct_name,struct_fields,"\t".repeat(depth)))
}

fn handle_enum(item_enum: ItemEnum, depth: usize, mut enumstruct: String) -> (Option<String>,Option<String>) {
    let enum_name = item_enum.clone().ident;
    let enum_variants = item_enum.clone().variants;
    enumstruct.push_str(&format!("enum {} {{\n{}\n}}",
        enum_name,
        enum_variants.into_iter().map(|variant| {
            format!("\t{},",variant.ident)}).collect::<Vec<String>>().join("\n"),       
    ));
    (Some(format!("{}",enum_name)),Some(enumstruct))
    // Some(format!("{} {{\n{}\n{}}}",
    //     enum_name,
    //     enum_variants.into_iter().map(|variant| {
    //         format!("{}{},","\t".repeat(depth+1),variant.ident)}).collect::<Vec<String>>().join("\n"),
    //     "\t".repeat(depth),
    // ))
}