use std::{collections::HashMap, f64};

use quote::ToTokens;
use syn::{Error, Expr, ExprField, ExprPath, FnArg, Ident, ItemFn, ItemStruct, Member, Pat, Result, Stmt, Type, TypePath, spanned::Spanned};

use crate::{
    Model,
    expression::{Graph, Node, binary::BinaryOp, unary::UnaryOp},
    model::{Function, VariableGraph},
};

// pub fn build_graph(f: &ItemFn, helpers: &HashMap<String, Graph>) -> Result<Graph> {}
//
// pub fn build_graph(
//     f: &ItemFn,
//     t: GraphType,
//     helpers: &HashMap<String, Graph>,
// ) -> Result<(Graph, Vec<String>, Vec<String>)> {
//     let (mut graph, mut map, parameter_order, data_order) = initialize(f, t).unwrap();
//     for statement in &f.block.stmts {
//         println!("{}", statement.to_token_stream());
//         match statement {
//             Stmt::Local(local) => {
//                 if let Pat::Ident(pattern_ident) = &local.pat {
//                     if let Some(init) = &local.init {
//                         let result = build_node(&mut graph, &map, &init.expr, helpers)?;
//                         map.insert(pattern_ident.ident.to_string(), result);
//                     }
//                 }
//             }
//             Stmt::Expr(expr, ..) => {
//                 let value = build_node(&mut graph, &map, expr, helpers)?;
//                 graph.value = Some(value);
//             }
//             _ => {
//                 return Err(Error::new_spanned(statement, "unsupported statement"));
//             }
//         }
//     }
//     if graph.value.is_none() {
//         return Err(Error::new_spanned(f, "function must return a value"));
//     }
//     Ok((graph, parameter_order, data_order))
// }
//
// pub enum GraphType {
//     Transformation(ItemStruct),
//     Model(ItemStruct, ItemStruct),
//     Helper,
// }
// fn initialize(
//     f: &ItemFn,
//     graph_type: GraphType,
// ) -> Result<(
//     Graph,
//     HashMap<String, *const Node>,
//     Vec<String>,
//     Vec<String>,
// )> {
//     let mut graph = Graph::new();
//     let mut map = HashMap::new();
//
//     let mut parameters_struct_names = Vec::new();
//     let mut data_struct_names = Vec::new();
//     let mut other_names = Vec::new();
//
//     for arg in &f.sig.inputs {
//         if let FnArg::Typed(pat_type) = arg {
//             let var_name = if let Pat::Ident(pat_ident) = &*pat_type.pat {
//                 pat_ident.ident.to_string()
//             } else {
//                 continue;
//             };
//
//             if let Type::Path(type_path) = &*pat_type.ty {
//                 if let Some(type_ident) = type_path.path.get_ident() {
//                     match type_ident.to_string().as_str() {
//                         "Parameters" => parameters_struct_names.push(var_name),
//                         "Data" => data_struct_names.push(var_name),
//                         "Float" => other_names.push(var_name),
//                         t => {
//                             let msg = match graph_type {
//                                 GraphType::Transformation(_) => "`Parameters`",
//                                 GraphType::Model(_, _) => "`Parameters` or `Data`",
//                                 GraphType::Helper => "`Float`",
//                             };
//                             return Err(syn::Error::new(
//                                 arg.span(),
//                                 format!("argument type `{}` is unsupported, must be {}", t, msg),
//                             ));
//                         }
//                     }
//                 }
//             }
//         }
//     }
//     let mut parameter_order = Vec::new();
//     let mut data_order = Vec::new();
//     match graph_type {
//         GraphType::Transformation(params) => {
//             if parameters_struct_names.len() != 1
//                 || data_struct_names.len() != 0
//                 || other_names.len() != 0
//             {
//                 return Err(syn::Error::new(
//                     f.sig.span(),
//                     format!(
//                         "transformation functions must take only one argument of type `self::Parameters`"
//                     ),
//                 ));
//             }
//             for parameter in params.fields.iter().map(|f| {
//                 format!(
//                     "{}.{}",
//                     parameters_struct_names[0],
//                     f.ident.clone().unwrap().to_string()
//                 )
//             }) {
//                 map.insert(
//                     parameter.clone(),
//                     graph.new_variable(parameter.clone(), true),
//                 );
//                 parameter_order.push(parameter);
//             }
//         }
//         GraphType::Model(params, data) => {
//             if parameters_struct_names.len() != 1
//                 || data_struct_names.len() != 1
//                 || other_names.len() != 0
//             {
//                 return Err(syn::Error::new(
//                     f.sig.span(),
//                     format!(
//                         "distribution functions must take exactly one argument each of type `Parameters` and `Data`"
//                     ),
//                 ));
//             }
//             for parameter in params.fields.iter().map(|f| {
//                 format!(
//                     "{}.{}",
//                     parameters_struct_names[0],
//                     f.ident.clone().unwrap().to_string()
//                 )
//             }) {
//                 map.insert(
//                     parameter.clone(),
//                     graph.new_variable(parameter.clone(), true),
//                 );
//                 parameter_order.push(parameter);
//             }
//             for d in data.fields.iter().map(|f| {
//                 format!(
//                     "{}.{}",
//                     data_struct_names[0],
//                     f.ident.clone().unwrap().to_string()
//                 )
//             }) {
//                 map.insert(d.clone(), graph.new_variable(d.clone(), false));
//                 data_order.push(d);
//             }
//         }
//         GraphType::Helper => {
//             if parameters_struct_names.len() != 0 || data_struct_names.len() != 0 {
//                 return Err(syn::Error::new(
//                     f.sig.span(),
//                     format!("helper functions must take only arguments of type `Float`"),
//                 ));
//             }
//             for argument in other_names {
//                 map.insert(argument.clone(), graph.new_variable(argument, true));
//             }
//         }
//     }
//     Ok((graph, map, parameter_order, data_order))
// }
//
// pub fn build_graph(
//     f: &ItemFn,
//     t: GraphType,
//     helpers: &HashMap<String, Graph>,
// ) -> Result<(Graph, Vec<String>, Vec<String>)> {
//     let (mut graph, mut map, parameter_order, data_order) = initialize(f, t).unwrap();
//     for statement in &f.block.stmts {
//         println!("{}", statement.to_token_stream());
//         match statement {
//             Stmt::Local(local) => {
//                 if let Pat::Ident(pattern_ident) = &local.pat {
//                     if let Some(init) = &local.init {
//                         let result = build_node(&mut graph, &map, &init.expr, helpers)?;
//                         map.insert(pattern_ident.ident.to_string(), result);
//                     }
//                 }
//             }
//             Stmt::Expr(expr, ..) => {
//                 let value = build_node(&mut graph, &map, expr, helpers)?;
//                 graph.value = Some(value);
//             }
//             _ => {
//                 return Err(Error::new_spanned(statement, "unsupported statement"));
//             }
//         }
//     }
//     if graph.value.is_none() {
//         return Err(Error::new_spanned(f, "function must return a value"));
//     }
//     Ok((graph, parameter_order, data_order))
// }

fn initialize(graph: &mut Graph, map: &mut HashMap<String, *const Node>, prefix: String, argument: &Option<*const VariableGraph>, parameter: bool) {
    match argument {
        Some(arg) => {
            let variable_graph = unsafe { &**arg };
            for sub_argument in &variable_graph.subgraphs {
                let new_prefix = format!("{}.{}", prefix, sub_argument.0);
                initialize(graph, map, new_prefix, sub_argument.1, parameter);
            }
        }
        None => {
            map.insert(prefix.clone(), graph.new_variable(prefix, parameter));
        }
    }
}

pub fn build_graph(function: &Function, model: &Model) -> Result<Graph> {
    let mut graph = Graph::new();
    println!("building graph for {:?}", function.name);
    let mut map = HashMap::new();
    let function_arguments = &function.argument_order;

    for arg in function_arguments {
        match function.argument_types.get(arg).unwrap().as_str() {
            "Float" => {
                initialize(&mut graph, &mut map, arg.clone(), &None, true);
            }
            other => {
                println!("Other: {}", other);
                println!("{:?}", model.structs.keys());
                initialize(&mut graph, &mut map, arg.clone(), &Some(&**model.structs.get(other).unwrap() as *const VariableGraph), other == "Parameter");
            }
        }
    }
    println!("completed initialization");

    let function_tokens = &function.tokens;

    for statement in &function_tokens.block.stmts {
        println!("{}", statement.to_token_stream());
        match statement {
            Stmt::Local(local) => {
                if let Pat::Ident(pattern_ident) = &local.pat {
                    if let Some(init) = &local.init {
                        let result = build_node(&mut graph, &map, &init.expr, model)?;
                        map.insert(pattern_ident.ident.to_string(), result);
                    }
                }
            }
            Stmt::Expr(expr, ..) => {
                let value = build_node(&mut graph, &map, &expr, model)?;
                graph.value = Some(value);
            }
            _ => {
                return Err(Error::new_spanned(statement, "unsupported statement"));
            }
        }
    }
    if graph.value.is_none() {
        return Err(Error::new(function_tokens.span(), "function must return a value"));
    }
    println!("completed graph for {}, {:?}", function.name, graph);
    Ok(graph)
}

fn build_node(graph: &mut Graph, map: &HashMap<String, *const Node>, expr: &Expr, model: &Model) -> Result<*const Node> {
    match expr {
        Expr::Binary(expr_bin) => {
            let left = build_node(graph, map, &expr_bin.left, model)?;
            let right = build_node(graph, map, &expr_bin.right, model)?;

            let binop = match &expr_bin.op {
                syn::BinOp::Add(_) => BinaryOp::Add,
                syn::BinOp::Sub(_) => BinaryOp::Sub,
                syn::BinOp::Mul(_) => BinaryOp::Mul,
                syn::BinOp::Div(_) => BinaryOp::Div,
                _ => {
                    return Err(syn::Error::new_spanned(&expr_bin.op, "operation not supported"));
                }
            };
            return Ok(graph.new_binary(binop, left, right));
        }

        Expr::Path(ExprPath { path, .. }) => {
            let segments: Vec<_> = path.segments.iter().collect();
            if segments.len() == 1 {
                Ok(*map.get(&path.segments[0].ident.to_string()).unwrap())
            } else if segments.len() == 2 && segments[0].ident == "Constants" {
                match segments[1].ident.to_string().as_str() {
                    "PI" => return Ok(graph.new_constant(f64::consts::PI)),
                    "E" => return Ok(graph.new_constant(f64::consts::E)),
                    _ => {
                        return Err(syn::Error::new_spanned(expr, format!("unsupported constant: {}", segments[1].ident.to_string())));
                    }
                }
            } else {
                Err(syn::Error::new_spanned(expr, format!("unsupported constant")))
            }
        }

        Expr::Field(f) => {
            let base_name = if let Expr::Path(base_path) = &*f.base {
                if let Some(ident) = base_path.path.get_ident() {
                    Some(ident.to_string())
                } else {
                    None
                }
            } else {
                None
            };
            match &f.member {
                Member::Named(field_ident) => {
                    let name = format!("{}.{}", base_name.unwrap(), field_ident.to_string());
                    Ok(*map.get(&name).unwrap())
                }
                Member::Unnamed(index) => {
                    let name = format!("{}.{}", base_name.unwrap(), index.index);

                    Ok(*map.get(&name).unwrap())
                }
            }
        }
        Expr::Lit(syn::ExprLit { lit, .. }) => match lit {
            syn::Lit::Float(f) => Ok(graph.new_constant(f.base10_parse::<f64>()?)),
            _ => Err(syn::Error::new_spanned(lit, "unsupported literal")),
        },
        Expr::Paren(inner) => build_node(graph, map, &inner.expr, model),
        Expr::Unary(expr_unary) => {
            if let syn::UnOp::Neg(_) = expr_unary.op {
                let argument = build_node(graph, map, &expr_unary.expr, model)?;
                return Ok(graph.new_unary(UnaryOp::Negative, argument));
            }
            Err(syn::Error::new_spanned(expr_unary, "unsupported unary operator (can only be '-', not '&', '*', or '!')"))
        }
        // Expr::Cast(expr_cast) => {
        //     let inner_node = build_node(graph, map, &expr_cast.expr)?;
        //     if let syn::Type::Path(type_path) = &*expr_cast.ty {
        //         let ident = &type_path.path.segments.last().unwrap().ident;
        //         let allowed = ["Float"];
        //         if !allowed.contains(&ident.to_string().as_str()) {
        //             return Err(syn::Error::new_spanned(ident, "unsupported cast type"));
        //         }
        //     }
        //
        //     Ok(inner_node)
        // }
        Expr::MethodCall(method_call) => {
            let method_name = method_call.method.to_string();
            if method_call.args.is_empty() {
                let unary_op = match method_name.as_str() {
                    "sin" => UnaryOp::Sin,
                    "cos" => UnaryOp::Cos,
                    "tan" => UnaryOp::Tan,
                    "exp" => UnaryOp::Exp,
                    "ln" => UnaryOp::Log,
                    _ => {
                        return Err(syn::Error::new_spanned(method_call, format!("unsupported method call: `{}`", method_name)));
                    }
                };
                let argument = build_node(graph, map, &method_call.receiver, model)?;
                return Ok(graph.new_unary(unary_op, argument));
            }

            if method_call.args.len() == 1 {
                let binop = match method_name.as_str() {
                    "powf" => BinaryOp::Pow,
                    _ => {
                        return Err(syn::Error::new_spanned(method_call, format!("unsupported method call: `{}`", method_name)));
                    }
                };
                let left = build_node(graph, map, &method_call.receiver, model)?;
                let right = build_node(graph, map, method_call.args.first().unwrap(), model)?;
                return Ok(graph.new_binary(binop, left, right));
            }

            Err(syn::Error::new_spanned(method_call, format!("unsupported method call: {}", method_name)))
        }
        Expr::Call(call) => {
            println!("call");
            if let Expr::Path(path) = &*call.func {
                println!("call2");
                let function_name = path.path.segments[0].ident.to_string();

                if model.functions.contains_key(&function_name) {
                    println!("123");
                    let helper = model.functions.get(&function_name).unwrap();
                    let exists = {
                        let h = helper.graph.borrow();
                        h.is_some()
                    };

                    if !exists {
                        let g = build_graph(helper, model)?;
                        helper.graph.replace(Some(g));
                        println!("reached");
                    }
                    let mut inputs = Vec::new();
                    for arg in &call.args {
                        inputs.push(build_node(graph, map, arg, model)?);
                    }
                    // let inputs = call
                    //     .args
                    //     .iter()
                    //     .map(|arg| {
                    //         let n = build_node(graph, map, arg, model);
                    //         if n.is_err() {
                    //             return n.err();
                    //         }
                    //         return n.ok();
                    //     })
                    //     .collect();
                    println!("{:?}", inputs);

                    let h = helper.graph.borrow();
                    let helper_graph = h.as_ref().unwrap();

                    println!("{:?}", helper_graph);
                    return Ok(graph.splice(helper_graph, inputs));
                }

                let unary_op = match function_name.as_str() {
                    "sin" => UnaryOp::Sin,
                    "cos" => UnaryOp::Cos,
                    "tan" => UnaryOp::Tan,
                    "exp" => UnaryOp::Exp,
                    "ln" => UnaryOp::Log,
                    _ => {
                        return Err(syn::Error::new_spanned(call, format!("unsupported function call: `{}`", function_name)));
                    }
                };
                let argument = build_node(graph, map, &call.args[0], model)?;
                return Ok(graph.new_unary(unary_op, argument));
            };
            Err(syn::Error::new_spanned(call, "unsupported function call"))
        }

        Expr::Return(ret) => {
            let Some(ret_expr) = &ret.expr else {
                return Err(syn::Error::new_spanned(ret, "`return` without value is unsupported"));
            };
            let result = build_node(graph, map, ret_expr, model)?;
            graph.value = Some(result);
            Ok(result)
        }
        _ => Err(syn::Error::new_spanned(expr, "unsupported expression")),
    }
}
//
// pub struct ModelArgs {
//     pub data: Vec<Ident>,
//     pub parameters: Vec<Ident>,
// }
//
// pub fn arguments(f: &ItemFn) -> ModelArgs {
//     let mut data = Vec::new();
//     let mut parameters = Vec::new();
//
//     for input in &f.sig.inputs {
//         if let FnArg::Typed(pat_type) = input {
//             let ident = if let Pat::Ident(pat_ident) = &*pat_type.pat {
//                 pat_ident.ident.clone()
//             } else {
//                 continue;
//             };
//
//             match &*pat_type.ty {
//                 Type::Path(type_path) => {
//                     let type_ident = type_path.path.segments.last().unwrap().ident.to_string();
//                     match type_ident.as_str() {
//                         "Parameter" => parameters.push(ident),
//                         "Data" => data.push(ident),
//                         _ => {}
//                     }
//                 }
//                 _ => {}
//             }
//         }
//     }
//
//     ModelArgs { data, parameters }
// }
