use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    hash::Hash,
};

use proc_macro2::Span;
use syn::{
    Field, FnArg, Item, ItemFn, ItemMod, ItemStruct, Pat, PatType, Result, ReturnType, Type,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

use quote::quote;

use crate::Graph;

pub struct Model {
    pub structs: HashMap<String, Box<VariableGraph>>,
    pub functions: HashMap<String, Function>,
    pub submodels: HashMap<String, Self>,
}

pub struct VariableGraph {
    pub name: String,
    pub subgraphs: HashMap<String, Option<*const VariableGraph>>,
}

impl VariableGraph {
    fn build(
        root: &ItemStruct,
        structs: &HashMap<String, ItemStruct>,
        graphs: &mut HashMap<String, Box<VariableGraph>>,
    ) -> Result<*const Self> {
        let name = root.ident.to_string();
        let mut subgraphs = HashMap::new();
        for field in &root.fields {
            let field_span = field.span();
            let field_name = field.ident.clone().unwrap().to_string();
            let field_ty = &field.ty;
            let field_type = quote!(#field_ty).to_string();
            match field_type.as_str() {
                "Float" => {
                    subgraphs.insert(field_name, None);
                }
                _ => {
                    if graphs.contains_key(&field_name) {
                        let graph = &**graphs.get(&field_name).unwrap() as *const VariableGraph;
                        subgraphs.insert(field_name, Some(graph));
                    } else {
                        if !structs.contains_key(&field_type) {
                            return Err(syn::Error::new(
                                field_span,
                                format!("unrecognized type: `{}`", field_type),
                            ));
                        }
                        let new_root = structs.get(&field_type).unwrap();
                        let graph = Self::build(new_root, structs, graphs)?;
                        subgraphs.insert(field_name, Some(graph));
                    }
                }
            }
        }
        graphs.insert(
            name.clone(),
            Box::new(Self {
                name: name.clone(),
                subgraphs,
            }),
        );
        return Ok(&**graphs.get(&name).unwrap() as *const VariableGraph);
    }
}

pub struct Function {
    pub name: String,
    pub argument_order: Vec<String>,
    pub argument_types: HashMap<String, String>,
    pub tokens: ItemFn,
    pub graph: RefCell<Option<Graph>>,
}

impl Model {
    fn get_types(f: &ItemFn) -> Result<(Vec<String>, HashMap<String, String>, String)> {
        let mut argument_order = Vec::new();
        let mut argument_types = HashMap::new();
        for arg in &f.sig.inputs {
            match arg {
                FnArg::Typed(pat_type) => match &*pat_type.ty {
                    Type::Path(type_path) => {
                        let segments = &type_path.path.segments;
                        let path_string = segments
                            .iter()
                            .map(|segment| segment.ident.to_string())
                            .collect::<Vec<String>>()
                            .join("::");
                        println!("{:?}", path_string);
                        let var_name = if let Pat::Ident(pat_ident) = &*pat_type.pat {
                            pat_ident.ident.to_string()
                        } else {
                            return Err(syn::Error::new(pat_type.pat.span(), "invalid argument"));
                        };
                        argument_order.push(var_name.clone());
                        argument_types.insert(var_name, path_string);
                    }
                    _ => {
                        return Err(syn::Error::new(pat_type.ty.span(), "invalid argument"));
                    }
                },
                FnArg::Receiver(_) => {
                    return Err(syn::Error::new(
                        f.sig.inputs.span(),
                        "self receivers are not allowed",
                    ));
                }
            }
        }
        let return_string = match &f.sig.output {
            ReturnType::Type(_, return_type) => match &**return_type {
                Type::Path(type_path) => {
                    let segments = &type_path.path.segments;
                    segments
                        .iter()
                        .map(|segment| segment.ident.to_string())
                        .collect::<Vec<String>>()
                        .join("::")
                }
                _ => {
                    return Err(syn::Error::new(
                        return_type.span(),
                        "function must have a return type",
                    ));
                }
            },
            ReturnType::Default => {
                return Err(syn::Error::new(
                    f.sig.ident.span(),
                    "function must have a return type",
                ));
            }
        };
        Ok((argument_order, argument_types, return_string))
    }

    fn new(span: Span, input: Vec<Item>, base_model: bool) -> Result<Self> {
        let mut function_tokens = HashMap::new();
        let mut module_tokens = HashMap::new();
        let mut struct_tokens = HashMap::new();

        // while !input.is_empty() {
        //     let item: Item = input.parse()?;
        for item in input {
            match item {
                Item::Fn(f) => {
                    let name = f.sig.ident.to_string();
                    function_tokens.insert(name, f);
                }
                Item::Mod(m) => {
                    let name = m.ident.to_string();
                    module_tokens.insert(name, m);
                }
                Item::Struct(s) => {
                    let name = s.ident.to_string();
                    struct_tokens.insert(name, s);
                }
                _ => {
                    return Err(syn::Error::new(item.span(), "unsupported model component"));
                }
            }
        }

        if base_model {
            if !struct_tokens.contains_key("Parameters") {
                return Err(syn::Error::new(
                    span,
                    "model must define struct `Parameters`",
                ));
            }
            if !struct_tokens.contains_key("Data") {
                return Err(syn::Error::new(span, "model must define struct `Data`"));
            }
            if !function_tokens.contains_key("distribution") {
                return Err(syn::Error::new(span, "model must define fn `distribution`"));
            }
            if !function_tokens.contains_key("generation") {
                return Err(syn::Error::new(span, "model must define fn `generation`"));
            }
        } else {
            if !struct_tokens.contains_key("Parameters") {
                return Err(syn::Error::new(
                    span,
                    "submodel must define struct `Parameters`",
                ));
            }
            if !function_tokens.contains_key("transformation") {
                return Err(syn::Error::new(
                    span,
                    "submodel must define fn `transformation`",
                ));
            }
        }

        let mut structs = HashMap::new();
        // struct_tokens
        //     .iter()
        //     .map(|(_, s)| VariableGraph::build(s, &struct_tokens, &mut structs))
        //     .collect::<Vec<_>>();
        for (_, s) in &struct_tokens {
            VariableGraph::build(s, &struct_tokens, &mut structs)?;
        }

        let mut functions = HashMap::new();
        for (name, function) in function_tokens {
            let (argument_order, argument_types, return_type) = Self::get_types(&function)?;
            if base_model {
                if name == "distribution" {
                    if argument_order.len() != 2
                        || !argument_types
                            .values()
                            .any(|s| s == "Parameters" || s == "self::Parameters")
                        || !argument_types
                            .values()
                            .any(|s| s == "Data" || s == "self::Data")
                    {
                        return Err(syn::Error::new(
                            function.sig.inputs.span(),
                            "`distribution` must take one argument of type `Parameters` and one of type `Data`",
                        ));
                    }
                    if return_type != "Float" {
                        return Err(syn::Error::new(
                            function.sig.output.span(),
                            "`distribution` return type must be `Float`",
                        ));
                    }
                } else if name == "generation" {
                    if argument_order.len() != 1
                        || !argument_types
                            .values()
                            .any(|s| s == "Parameters" || s == "self::Parameters")
                    {
                        return Err(syn::Error::new(
                            function.sig.inputs.span(),
                            "`generation` must take one argument of type `Parameters`",
                        ));
                    }
                    if return_type != "Data" && return_type != "self::Data" {
                        return Err(syn::Error::new(
                            function.sig.output.span(),
                            "`generation` return type must be `Data`",
                        ));
                    }
                }
            } else {
                if name == "transformation" {
                    if argument_order.len() != 1
                        || !argument_types
                            .values()
                            .any(|s| s == "Parameters" || s == "self::Parameters")
                    {
                        return Err(syn::Error::new(
                            function.sig.inputs.span(),
                            "`transformation` must take one argument of type `self::Parameters`",
                        ));
                    }
                    if return_type != "super::Parameters" {
                        return Err(syn::Error::new(
                            function.sig.output.span(),
                            "`transformation` return type must be `super::Parameters`",
                        ));
                    }
                }
            }
            let f = Function {
                name: name.clone(),
                argument_order,
                argument_types,
                tokens: function,
                graph: RefCell::new(None),
            };
            functions.insert(name, f);
        }

        let mut submodels = HashMap::new();
        let mut errors = Vec::new();

        for (name, module) in module_tokens {
            let content = match &module.content {
                Some((_, items)) => items,
                None => {
                    let e = syn::Error::new_spanned(module, "submodels must be declared inline");
                    errors.push(e.clone());
                    continue;
                }
            };
            let submodel = Self::new(module.span(), content.clone(), false);
            if submodel.is_ok() {
                submodels.insert(name, submodel?);
            } else {
                errors.push(submodel.err().unwrap());
                continue;
            }
        }
        if errors.len() != 0 {
            let mut error = errors.swap_remove(0);
            for e in errors {
                error.combine(e);
            }
            return Err(error);
        }

        Ok(Self {
            structs,
            functions,
            submodels,
        })
    }
}

impl Parse for Model {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            let item = input.parse()?;
            items.push(item);
        }
        return Model::new(input.span(), items, true);
    }
}

//
// pub struct Model {
//     pub(crate) parameters: ItemStruct,
//     pub(crate) data: ItemStruct,
//     pub(crate) distribution: ItemFn,
//     pub(crate) generation: ItemFn,
//     pub(crate) helpers: Vec<ItemFn>,
//     pub(crate) submodels: Vec<ItemMod>,
// }
//
// impl Parse for Model {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let mut parameters = None;
//         let mut data = None;
//         let mut distribution = None;
//         let mut generation = None;
//         let mut helpers = Vec::new();
//         let mut submodels = Vec::new();
//
//         while !input.is_empty() {
//             let item: Item = input.parse()?;
//
//             match item {
//                 Item::Fn(f) => {
//                     let name = f.sig.ident.to_string();
//                     match name.as_str() {
//                         "distribution" => {
//                             if distribution.is_some() {
//                                 return Err(syn::Error::new(
//                                     f.sig.ident.span(),
//                                     "duplicate function definition for `distribution`",
//                                 ));
//                             }
//                             match &f.sig.output {
//                                 ReturnType::Type(_, return_type) => match &**return_type {
//                                     Type::Path(type_path) => {
//                                         let segments = &type_path.path.segments;
//                                         if segments.len() != 1 || segments[0].ident != "Float" {
//                                             return Err(syn::Error::new(
//                                                 return_type.span(),
//                                                 "distribution function must return `Float`",
//                                             ));
//                                         }
//                                     }
//                                     _ => {
//                                         return Err(syn::Error::new(
//                                             return_type.span(),
//                                             "distribution function must return `Float`",
//                                         ));
//                                     }
//                                 },
//                                 ReturnType::Default => {
//                                     return Err(syn::Error::new(
//                                         f.sig.ident.span(),
//                                         "distribution function must return `Float`",
//                                     ));
//                                 }
//                             }
//                             if f.sig.inputs.len() != 1 {
//                                 return Err(syn::Error::new(
//                                     f.sig.inputs.span(),
//                                     "distribution function must take one argument of type `Parameters`",
//                                 ));
//                             }
//                             match &f.sig.inputs[0] {
//                                 FnArg::Typed(PatType { ty, .. }) => match &**ty {
//                                     Type::Path(type_path) => {
//                                         let segments = &type_path.path.segments;
//                                         if segments.len() != 1 || segments[0].ident != "Parameters"
//                                         {
//                                             return Err(syn::Error::new(
//                                                 ty.span(),
//                                                 "argument must be of type `Parameters`",
//                                             ));
//                                         } else if segments.len() != 2
//                                             || segments[0].ident != "self"
//                                             || segments[1].ident != "Parameters"
//                                         {
//                                             return Err(syn::Error::new(
//                                                 ty.span(),
//                                                 "argument must be of type `self::Parameters`",
//                                             ));
//                                         }
//                                     }
//                                     _ => {
//                                         return Err(syn::Error::new(
//                                             ty.span(),
//                                             "argument must be of type `Parameters`",
//                                         ));
//                                     }
//                                 },
//                                 FnArg::Receiver(_) => {
//                                     return Err(syn::Error::new(
//                                         f.sig.inputs.span(),
//                                         "`distribution` must not have a self receiver",
//                                     ));
//                                 }
//                             }
//
//                             distribution = Some(f);
//                         }
//                         "generation" => {
//                             if generation.is_some() {
//                                 return Err(syn::Error::new(
//                                     f.sig.ident.span(),
//                                     "duplicate function definition for `generation`",
//                                 ));
//                             }
//                             match &f.sig.output {
//                                 ReturnType::Type(_, return_type) => match &**return_type {
//                                     Type::Path(type_path) => {
//                                         let segments = &type_path.path.segments;
//                                         if segments.len() != 1 || segments[0].ident != "Data" {
//                                             return Err(syn::Error::new(
//                                                 return_type.span(),
//                                                 "generation function must return `Data`",
//                                             ));
//                                         }
//                                     }
//                                     _ => {
//                                         return Err(syn::Error::new(
//                                             return_type.span(),
//                                             "generation function must return `Data`",
//                                         ));
//                                     }
//                                 },
//                                 ReturnType::Default => {
//                                     return Err(syn::Error::new(
//                                         f.sig.ident.span(),
//                                         "generation function must return `Data`",
//                                     ));
//                                 }
//                             }
//                             if f.sig.inputs.len() != 1 {
//                                 return Err(syn::Error::new(
//                                     f.sig.inputs.span(),
//                                     "generation function must take one argument of type `Parameters`",
//                                 ));
//                             }
//                             match &f.sig.inputs[0] {
//                                 FnArg::Typed(PatType { ty, .. }) => match &**ty {
//                                     Type::Path(type_path) => {
//                                         let segments = &type_path.path.segments;
//                                         if segments.len() != 1 || segments[0].ident != "Parameters"
//                                         {
//                                             return Err(syn::Error::new(
//                                                 ty.span(),
//                                                 "argument must be of type `Parameters`",
//                                             ));
//                                         } else if segments.len() != 2
//                                             || segments[0].ident != "self"
//                                             || segments[1].ident != "Parameters"
//                                         {
//                                             return Err(syn::Error::new(
//                                                 ty.span(),
//                                                 "argument must be of type `self::Parameters`",
//                                             ));
//                                         }
//                                     }
//                                     _ => {
//                                         return Err(syn::Error::new(
//                                             ty.span(),
//                                             "argument must be of type `Parameters`",
//                                         ));
//                                     }
//                                 },
//                                 FnArg::Receiver(_) => {
//                                     return Err(syn::Error::new(
//                                         f.sig.inputs.span(),
//                                         "`generation` must not have a self receiver",
//                                     ));
//                                 }
//                             }
//                             generation = Some(f);
//                         }
//                         _ => {
//                             helpers.push(f);
//                         }
//                     }
//                 }
//                 Item::Struct(s) => match s.ident.to_string().as_str() {
//                     "Parameters" => {
//                         if parameters.is_some() {
//                             return Err(syn::Error::new(
//                                 s.ident.span(),
//                                 "duplicate struct definition for `Parameters`",
//                             ));
//                         }
//                         parameters = Some(s);
//                     }
//                     "Data" => {
//                         if data.is_some() {
//                             return Err(syn::Error::new(
//                                 s.ident.span(),
//                                 "duplicate struct definition for `Data`",
//                             ));
//                         }
//                         data = Some(s);
//                     }
//                     name => {
//                         return Err(syn::Error::new(
//                             s.ident.span(),
//                             format!("unsupported struct definition for `{}`", name),
//                         ));
//                     }
//                 },
//                 Item::Mod(m) => {
//                     submodels.push(m);
//                 }
//                 _ => {
//                     return Err(syn::Error::new(item.span(), "unsupported model component"));
//                 }
//             }
//         }
//
//         let parameters = parameters
//             .ok_or_else(|| syn::Error::new(input.span(), "missing required struct `Parameters`"))?;
//         let data =
//             data.ok_or_else(|| syn::Error::new(input.span(), "missing required struct `Data`"))?;
//         let distribution = distribution.ok_or_else(|| {
//             syn::Error::new(input.span(), "missing required function `distribution`")
//         })?;
//         let generation = generation.ok_or_else(|| {
//             syn::Error::new(input.span(), "missing required function `generation`")
//         })?;
//
//         Ok(Model {
//             distribution,
//             parameters,
//             data,
//             generation,
//             helpers,
//             submodels,
//         })
//     }
// }
//
// pub struct SubModel {
//     pub(crate) transformation: ItemFn,
//     pub(crate) parameters: ItemStruct,
// }
//
// impl Parse for SubModel {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let mut transformation = None;
//         let mut parameters = None;
//
//         while !input.is_empty() {
//             let item: Item = input.parse()?;
//
//             match item {
//                 Item::Struct(s) => match s.ident.to_string().as_str() {
//                     "Parameters" => {
//                         if parameters.is_some() {
//                             return Err(syn::Error::new(
//                                 s.ident.span(),
//                                 "duplicate struct definition for `Parameters`",
//                             ));
//                         }
//                         parameters = Some(s);
//                     }
//                     name => {
//                         return Err(syn::Error::new(
//                             s.ident.span(),
//                             format!("unsupported struct definition for `{}`", name),
//                         ));
//                     }
//                 },
//
//                 Item::Fn(f) => {
//                     let name = f.sig.ident.to_string();
//                     match name.as_str() {
//                         "transform" => {
//                             if transformation.is_some() {
//                                 return Err(syn::Error::new(
//                                     f.sig.ident.span(),
//                                     "duplicate function definition for `transform`",
//                                 ));
//                             }
//
//                             match &f.sig.output {
//                                 ReturnType::Type(_, return_type) => match &**return_type {
//                                     Type::Path(type_path) => {
//                                         let segments: Vec<_> =
//                                             type_path.path.segments.iter().collect();
//                                         if segments.len() != 2
//                                             || segments[0].ident != "super"
//                                             || segments[1].ident != "Parameters"
//                                         {
//                                             return Err(syn::Error::new(
//                                                 return_type.span(),
//                                                 "must return `super::Parameters`",
//                                             ));
//                                         }
//                                     }
//                                     _ => {
//                                         return Err(syn::Error::new(
//                                             return_type.span(),
//                                             "must return `super::Parameters`",
//                                         ));
//                                     }
//                                 },
//                                 ReturnType::Default => {
//                                     return Err(syn::Error::new(
//                                         f.sig.ident.span(),
//                                         "must return `super::Parameters`",
//                                     ));
//                                 }
//                             }
//                             if f.sig.inputs.len() != 1 {
//                                 return Err(syn::Error::new(
//                                     f.sig.inputs.span(),
//                                     "must take exactly one argument",
//                                 ));
//                             }
//                             for arg in &f.sig.inputs {
//                                 match arg {
//                                     FnArg::Typed(PatType { ty, .. }) => match &**ty {
//                                         Type::Path(type_path) => {
//                                             let segments: Vec<_> =
//                                                 type_path.path.segments.iter().collect();
//                                             if segments.len() == 1 {
//                                                 if segments[0].ident != "Parameters" {
//                                                     return Err(syn::Error::new(
//                                                         ty.span(),
//                                                         "argument must be of type `Parameters`",
//                                                     ));
//                                                 }
//                                             } else if segments.len() != 2
//                                                 || segments[0].ident != "self"
//                                                 || segments[1].ident != "Parameters"
//                                             {
//                                                 return Err(syn::Error::new(
//                                                     ty.span(),
//                                                     "argument must be of type `self::Parameters`",
//                                                 ));
//                                             }
//                                         }
//                                         _ => {
//                                             return Err(syn::Error::new(
//                                                 ty.span(),
//                                                 "argument must be of type `self::Parameters`",
//                                             ));
//                                         }
//                                     },
//                                     FnArg::Receiver(_) => {
//                                         return Err(syn::Error::new(
//                                             arg.span(),
//                                             "`transform` must not have a self receiver",
//                                         ));
//                                     }
//                                 }
//                             }
//
//                             transformation = Some(f);
//                         }
//                         _ => {
//                             return Err(syn::Error::new(
//                                 f.sig.ident.span(),
//                                 format!(
//                                     "unsupported function definition for `{}`; accepted functions are `transform`",
//                                     name,
//                                 ),
//                             ));
//                         }
//                     }
//                 }
//                 _ => {
//                     return Err(syn::Error::new(
//                         item.span(),
//                         "unsupported submodel component; only `fn transform(...)` and `struct Parameters {...}` may be defined here",
//                     ));
//                 }
//             }
//         }
//
//         let transformation = transformation.ok_or_else(|| {
//             syn::Error::new(input.span(), "missing required function `transform`")
//         })?;
//
//         let parameters = parameters
//             .ok_or_else(|| syn::Error::new(input.span(), "missing required struct `Parameters`"))?;
//
//         Ok(SubModel {
//             transformation,
//             parameters,
//         })
//     }
// }
