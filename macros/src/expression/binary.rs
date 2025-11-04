use std::rc::Rc;

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::expression::variable::Variable;
use crate::expression::{Graph, Node, NodeType};
use crate::expression::{Unary, UnaryOp};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub(crate) struct Binary {
    pub(crate) operation: BinaryOp,
    pub(crate) left: Rc<Node>,
    pub(crate) right: Rc<Node>,
}

impl Binary {
    pub(crate) fn new(graph: &mut Graph, operation: BinaryOp, left: Rc<Node>, right: Rc<Node>) -> Rc<Node> {
        match operation {
            BinaryOp::Add => {
                if *left == 0.0 {
                    return right;
                } else if *right == 0.0 {
                    return left;
                }
            }
            BinaryOp::Sub => {
                if *left == 0.0 {
                    return Unary::new(graph, UnaryOp::Negative, right);
                } else if *right == 0.0 {
                    return left;
                } else if *left == *right {
                    return graph.new_constant(0.0);
                }
            }
            BinaryOp::Mul => {
                if *left == 0.0 || *right == 0.0 {
                    return graph.new_constant(0.0);
                } else if *left == 1.0 {
                    return right;
                } else if *right == 1.0 {
                    return left;
                } else if *left == -1.0 {
                    return graph.new_unary(UnaryOp::Negative, right);
                } else if *right == -1.0 {
                    return graph.new_unary(UnaryOp::Negative, left);
                }
            }
            BinaryOp::Div => {
                if *left == 0.0 {
                    return graph.new_constant(0.0);
                } else if *right == 1.0 {
                    return left;
                } else if *right == -1.0 {
                    return graph.new_unary(UnaryOp::Negative, left);
                } else if *right == 0.0 {
                    panic!("attempted to divide by zero",);
                }
            }
            BinaryOp::Pow => {
                if *left == 0.0 {
                    return graph.new_constant(0.0);
                } else if *right == 0.0 {
                    return graph.new_constant(1.0);
                } else if *right == 1.0 {
                    return left;
                } else if *right == 2.0 {
                    return Self::new(graph, BinaryOp::Mul, left.clone(), left.clone());
                }
            }
        }
        let binary = Self { operation, left, right };
        graph.insert(Node::new(NodeType::Binary(binary)))
    }

    pub(crate) fn differentiate(&self, graph: &mut Graph, variable: &Variable) -> Rc<Node> {
        let left_deriv = graph.differentiate(&self.left, variable);
        let right_deriv = graph.differentiate(&self.right, variable);

        match self.operation {
            BinaryOp::Add => Self::new(graph, BinaryOp::Add, left_deriv, right_deriv),
            BinaryOp::Sub => Self::new(graph, BinaryOp::Sub, left_deriv, right_deriv),
            BinaryOp::Mul => {
                let left = Self::new(graph, BinaryOp::Mul, left_deriv, self.right.clone());
                let right = Self::new(graph, BinaryOp::Mul, self.left.clone(), right_deriv);
                Self::new(graph, BinaryOp::Add, left, right)
            }
            BinaryOp::Div => {
                let left = Self::new(graph, BinaryOp::Mul, left_deriv, self.right.clone());
                let right = Self::new(graph, BinaryOp::Mul, self.left.clone(), right_deriv);
                let numerator = Self::new(graph, BinaryOp::Sub, left, right);
                let denominator = Self::new(graph, BinaryOp::Mul, self.right.clone(), self.right.clone());
                Self::new(graph, BinaryOp::Div, numerator, denominator)
            }
            BinaryOp::Pow => {
                if let NodeType::Constant(c) = &self.right.interior {
                    let new_exp = graph.new_constant(c.value - 1.0);
                    let new = Self::new(graph, BinaryOp::Pow, self.left.clone(), new_exp);
                    let deriv = Self::new(graph, BinaryOp::Mul, self.right.clone(), new);
                    Self::new(graph, BinaryOp::Mul, left_deriv, deriv)
                } else {
                    panic!("non-constant exponents are not yet supported");
                }
            }
        }
    }
}

impl BinaryOp {
    pub fn cost(&self) -> usize {
        match &self {
            Self::Add => 3,
            Self::Sub => 3,
            Self::Mul => 5,
            Self::Div => 20,
            Self::Pow => 100,
        }
    }

    pub fn generate_rust(&self, result: Ident, left_value: Ident, right_value: Ident) -> TokenStream {
        match &self {
            Self::Add => quote! { let #result = #left_value + #right_value; },
            Self::Sub => quote! { let #result = #left_value - #right_value; },
            Self::Mul => quote! { let #result = #left_value * #right_value; },
            Self::Div => quote! { let #result = #left_value / #right_value; },
            Self::Pow => quote! { let #result = #left_value.powf(#right_value as f64); },
        }
    }
}
