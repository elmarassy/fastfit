use std::collections::HashMap;

use crate::{
    Model,
    expression::{Graph, Node, NodeType, binary::BinaryOp, constant::Constant, unary::UnaryOp},
    model::VariableGraph,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

pub fn translate_rust(
    graph: &Graph,
    fn_name: String,
    gradient: bool,
    hessian: bool,
) -> TokenStream {
    let mut num_params: usize = 0;
    let mut num_data: usize = 0;

    let eval_order = graph.order();

    let node_name = |node: *const Node| format_ident!("v{}", node as usize);

    let code: Vec<_> = eval_order
        .iter()
        .map(|&node| {
            let result_name = node_name(node);

            match &unsafe { &*node }.interior {
                NodeType::Constant(number) => {
                    let value = number.value;
                    quote! { let #result_name = #value; }
                }

                NodeType::Variable(variable) => {
                    let name = &variable.name;
                    if variable.parameter {
                        num_params += 1;
                        let parameter_index = variable.index;
                        quote! { let #result_name = parameters[#parameter_index]; }
                    } else {
                        num_data += 1;
                        let data_index = variable.index;
                        quote! { let #result_name = data[#data_index]; }
                    }
                }
                NodeType::Unary(u) => u
                    .operation
                    .generate_rust(result_name, node_name(u.argument)),
                NodeType::Binary(b) => {
                    b.operation
                        .generate_rust(result_name, node_name(b.left), node_name(b.right))
                }
                NodeType::Collection(_) => {
                    panic!(
                        "unable to generate rust code, collections should not appear in final graph"
                    );
                }
            }
        })
        .collect();
    let parameters = quote! {
        parameters: [Float; #num_params]
    };
    let data = quote! {
        data: [Float; #num_params]
    };

    let final_value_name = node_name(graph.value.unwrap());
    let fn_name = syn::Ident::new(&fn_name, Span::call_site());
    if gradient {
        let gradient_names = graph
            .gradient
            .iter()
            .map(|&id| node_name(id))
            .collect::<Vec<Ident>>();
        if hessian {
            let hessian_names = graph
                .hessian
                .iter()
                .map(|&id| node_name(id))
                .collect::<Vec<Ident>>();
            let num_hess = num_params * (num_params + 1) / 2 as usize;
            let signature = quote! {
                pub fn #fn_name(#parameters, #data) -> (f64, [f64; #num_params], [f64; #num_hess])
            };
            quote! {
                #signature {
                    #(#code)*
                    let gradient = [#(#gradient_names),*];
                    let hessian = [#(#hessian_names),*];
                    (#final_value_name, gradient, hessian)
                }
            }
            .into()
        } else {
            let signature = quote! {
                pub fn #fn_name(#parameters, #data) -> (f64, [f64; #num_params])
            };
            quote! {
                #signature {
                    #(#code)*
                    let gradient = [#(#gradient_names),*];
                    (#final_value_name, gradient)
                }
            }
            .into()
        }
    } else {
        if hessian {
            let hessian_names = graph
                .hessian
                .iter()
                .map(|&id| node_name(id))
                .collect::<Vec<Ident>>();
            let num_hess = num_params * (num_params + 1) / 2 as usize;
            let signature = quote! {
                pub fn #fn_name(#parameters, #data) -> (f64, [f64; #num_hess])
            };
            quote! {
                #signature {
                    #(#code)*
                    let hessian = [#(#hessian_names),*];
                    (#final_value_name, hessian)
                }
            }
            .into()
        } else {
            let signature = quote! {
                pub fn #fn_name(#parameters, #data) -> f64
            };
            quote! {
                #signature {
                    #(#code)*
                    #final_value_name
                }
            }
            .into()
        }
    }
}
