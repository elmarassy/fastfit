use std::rc::Rc;

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::expression::{
    Graph, Node, NodeType, Variable,
    binary::{Binary, BinaryOp},
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum UnaryOp {
    Negative,
    Exp,
    Log,
    Sin,
    Cos,
    Tan,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub(crate) struct Unary {
    pub(crate) operation: UnaryOp,
    pub(crate) argument: *const Node,
}

impl Unary {
    pub(crate) fn new(graph: &mut Graph, operation: UnaryOp, argument: *const Node) -> *const Node {
        let argref = &unsafe { &*argument }.interior;

        match operation {
            UnaryOp::Exp => {
                if let NodeType::Unary(u) = argref {
                    if let UnaryOp::Log = u.operation {
                        return u.argument.clone();
                    }
                }
            }
            UnaryOp::Log => {
                if let NodeType::Unary(u) = argref {
                    if let UnaryOp::Exp = u.operation {
                        return u.argument.clone();
                    }
                }
            }
            UnaryOp::Negative => {
                if let NodeType::Unary(u) = argref {
                    if let UnaryOp::Negative = u.operation {
                        return u.argument.clone();
                    }
                } else if let NodeType::Constant(c) = argref {
                    return graph.new_constant(-c.value);
                }
            }
            _ => {}
        }

        let base = Unary { operation, argument };
        return graph.insert(Node::new(NodeType::Unary(base)));
    }

    pub(crate) fn differentiate(&self, graph: &mut Graph, variable: &Variable) -> *const Node {
        let arg_deriv = graph.differentiate(self.argument, variable);
        match self.operation {
            UnaryOp::Negative => Self::new(graph, UnaryOp::Negative, arg_deriv),
            UnaryOp::Exp => {
                let exp = Self::new(graph, UnaryOp::Exp, self.argument.clone());
                Binary::new(graph, BinaryOp::Mul, arg_deriv, exp)
            }
            UnaryOp::Log => Binary::new(graph, BinaryOp::Div, arg_deriv, self.argument.clone()),
            UnaryOp::Sin => {
                let cos = Self::new(graph, UnaryOp::Cos, self.argument.clone());
                Binary::new(graph, BinaryOp::Mul, arg_deriv, cos)
            }
            UnaryOp::Cos => {
                let sin = Self::new(graph, UnaryOp::Sin, self.argument.clone());
                let negative_sin = Self::new(graph, UnaryOp::Negative, sin);
                Binary::new(graph, BinaryOp::Mul, arg_deriv, negative_sin)
            }
            UnaryOp::Tan => {
                let cos = Self::new(graph, UnaryOp::Cos, self.argument.clone());
                let p = graph.new_constant(-2.0);
                let sec2 = Binary::new(graph, BinaryOp::Pow, cos, p);
                Binary::new(graph, BinaryOp::Mul, arg_deriv, sec2)
            }
        }
    }
}
impl UnaryOp {
    pub fn cost(&self) -> usize {
        match &self {
            Self::Negative => 3,
            Self::Sin => 100,
            Self::Cos => 100,
            Self::Tan => 100,
            Self::Exp => 100,
            Self::Log => 100,
        }
    }
    pub fn generate_rust(&self, result: Ident, argument_value: Ident) -> TokenStream {
        match &self {
            Self::Negative => quote! { let #result = -#argument_value; },
            Self::Sin => quote! { let #result = #argument_value.sin(); },
            Self::Cos => quote! { let #result = #argument_value.cos(); },
            Self::Tan => quote! { let #result = #argument_value.tan(); },
            Self::Exp => quote! { let #result = #argument_value.exp(); },
            Self::Log => quote! { let #result = #argument_value.ln(); },
        }
    }
}
