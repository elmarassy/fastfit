use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemMod, parse_macro_input, spanned::Spanned};

#[allow(dead_code)]
mod expression;
mod model;
mod parse;
mod translate;

use model::Model;

use crate::expression::{Graph, unary::UnaryOp};

extern crate proc_macro;

fn generate_code(graph: &mut Graph) -> proc_macro2::TokenStream {
    let dist = translate::translate_rust(graph, "_dist".to_string(), false, false);

    let log = graph.new_unary(UnaryOp::Log, graph.value.clone().unwrap());
    graph.value = Some(graph.new_unary(UnaryOp::Negative, log));

    let likelihood = translate::translate_rust(graph, "_likelihood".to_string(), false, false);

    graph.compute_gradient();
    let gradient = translate::translate_rust(graph, "_grad".to_string(), true, false);

    graph.compute_hessian();
    let hessian = translate::translate_rust(graph, "_hess".to_string(), true, true);

    quote! {
        #dist
        #likelihood
        #gradient
        #hessian
    }
    .into()
}

fn create_submodel(base_graph: &Graph, name: &String, submodel: &Model, model: &Model) -> proc_macro2::TokenStream {
    todo!()
}

#[proc_macro_attribute]
pub fn define_model(_attr: TokenStream, module: TokenStream) -> TokenStream {
    let module: syn::ItemMod = parse_macro_input!(module as syn::ItemMod);
    let content = match &module.content {
        Some((_, items)) => items,
        None => {
            return syn::Error::new_spanned(module, "#[define_model] can only be used on module declarations").to_compile_error().into();
        }
    };
    println!("started");
    let model_name = module.ident;
    let model = match syn::parse::<Model>(quote! { #(#content)* }.into()) {
        Ok(model) => model,
        Err(e) => return e.to_compile_error().into(),
    };
    println!("made model");
    println!("{:?}", model.functions.keys());

    let mut base_graph = match parse::build_graph(model.functions.get("distribution").unwrap(), &model) {
        Ok(g) => g,
        Err(e) => {
            return e.into_compile_error().into();
        }
    };
    println!("made graph");
    let mut submodel_code = Vec::new();
    for submodel in &model.submodels {
        submodel_code.push(create_submodel(&base_graph, submodel.0, submodel.1, &model));
    }

    let model_code = generate_code(&mut base_graph);
    let output = quote! {
        pub mod #model_name {
            use super::*;
            type Float = f64;
            #(#content)*
            #model_code
            #(#submodel_code)*
        }
    };
    println!("{}", output);
    output.into()
}
//
// #[proc_macro_attribute]
// pub fn define_model2(_attr: TokenStream, module: TokenStream) -> TokenStream {
//     let module: syn::ItemMod = parse_macro_input!(module as syn::ItemMod);
//
//     let content = match &module.content {
//         Some((_, items)) => items,
//         None => {
//             return syn::Error::new_spanned(
//                 module,
//                 "#[define_model] can only be used on inline module declarations",
//             )
//             .to_compile_error()
//             .into();
//         }
//     };
//
//     let model_name = module.ident;
//     let model = match syn::parse::<Model>(quote! { #(#content)* }.into()) {
//         Ok(m) => m,
//         Err(e) => return e.to_compile_error().into(),
//     };
//     println!("1");
//     let distribution = model.distribution;
//     let parameters = model.parameters;
//     let data = model.data;
//     let helper_functions = model.helpers;
//     let mut helpers = HashMap::new();
//     for helper in &helper_functions {
//         let name = helper.sig.ident.to_string();
//         let graph = parse::build_graph(helper, GraphType::Helper, &helpers)
//             .ok()
//             .unwrap();
//         helpers.insert(name, graph.0);
//     }
//     // let args = parse::arguments(&distribution);
//     // let data_names = args.data.iter().map(|data| data.to_string()).collect();
//     // let parameter_names = args
//     //     .parameters
//     //     .iter()
//     //     .map(|param| param.to_string())
//     //     .collect();
//
//     // println!("2");
//     // let parameter_names = model
//     //     .parameters
//     //     .fields
//     //     .iter()
//     //     .map(|f| f.ident.clone().unwrap().to_string())
//     //     .collect();
//     //
//     // println!("3");
//     // let data_names = model
//     //     .parameters
//     //     .fields
//     //     .iter()
//     //     .map(|f| f.ident.clone().unwrap().to_string())
//     //     .collect();
//
//     let (mut distribution_graph, parameter_names, data_names) = match parse::build_graph(
//         &distribution,
//         GraphType::Model(parameters.clone(), data.clone()),
//         &helpers,
//     ) {
//         Ok(graph) => graph,
//         Err(e) => {
//             return syn::Error::new_spanned(distribution, e.to_string())
//                 .to_compile_error()
//                 .into();
//         }
//     };
//
//     // let mut distribution_graph = match parse::build_graph(&distribution, &args) {
//     //     Ok(graph) => graph,
//     //     Err(e) => {
//     //         return syn::Error::new_spanned(distribution, e.to_string())
//     //             .to_compile_error()
//     //             .into();
//     //     }
//     // };
//
//     let backend_distribution = translate::translate(
//         &distribution_graph,
//         &parameter_names,
//         &data_names,
//         Ident::new("_distribution", distribution.span()),
//         false,
//         false,
//     );
//
//     let log_likelihood =
//         distribution_graph.new_unary(UnaryOp::Log, distribution_graph.value.unwrap());
//     distribution_graph.value =
//         Some(distribution_graph.new_unary(UnaryOp::Negative, log_likelihood));
//
//     distribution_graph.compute_gradient();
//     distribution_graph.compute_hessian();
//
//     let backend_likelihood = translate::translate(
//         &distribution_graph,
//         &parameter_names,
//         &data_names,
//         Ident::new("_likelihood", distribution.span()),
//         false,
//         false,
//     );
//     let backend_likelihood_grad = translate::translate(
//         &distribution_graph,
//         &parameter_names,
//         &data_names,
//         Ident::new("_likelihood_grad", distribution.span()),
//         true,
//         false,
//     );
//     let backend_likelihood_grad_hess = translate::translate(
//         &distribution_graph,
//         &parameter_names,
//         &data_names,
//         Ident::new("_likelihood_grad_hess", distribution.span()),
//         true,
//         true,
//     );
//
//     let output = quote! {
//         pub mod #model_name {
//             use super::*;
//             type Float = f64;
//             #parameters
//             #data
//             #(#helper_functions)*
//             #distribution
//             #backend_distribution
//             #backend_likelihood
//             #backend_likelihood_grad
//             #backend_likelihood_grad_hess
//         }
//     };
//     output.into()
// }
