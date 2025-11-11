pub(crate) mod binary;
pub(crate) mod collection;
pub(crate) mod constant;
pub(crate) mod unary;
pub(crate) mod variable;

use std::collections::{HashMap, HashSet};

use binary::Binary;
use collection::Collection;
use constant::Constant;
use unary::Unary;
use variable::Variable;

use crate::expression::binary::BinaryOp;
use crate::expression::collection::{Array, Struct, Tuple};
use crate::expression::unary::UnaryOp;

#[derive(Debug, Eq, Hash, PartialEq)]
pub(crate) enum NodeType {
    Binary(Binary),
    Constant(Constant),
    Unary(Unary),
    Variable(Variable),
    Collection(Collection),
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub(crate) struct Node {
    pub(crate) interior: NodeType,
    pub(crate) parameters: bool,
    pub(crate) data: bool,
    pub(crate) cost: usize,
}

impl Node {
    pub fn new(interior: NodeType) -> Self {
        unsafe {
            match &interior {
                NodeType::Binary(b) => {
                    let parameters = (*b.left).parameters || (*b.right).parameters;
                    let data = (*b.left).data || (*b.right).data;
                    let cost = (*b.left).cost + (*b.right).cost + b.operation.cost();
                    Self { interior, parameters, data, cost }
                }
                NodeType::Collection(_) => Self { interior, parameters: false, data: false, cost: 0 },
                NodeType::Constant(_) => Self { interior, parameters: false, data: false, cost: 1 },
                NodeType::Unary(u) => {
                    let parameters = (*u.argument).parameters;
                    let data = (*u.argument).data;
                    let cost = (*u.argument).cost + u.operation.cost();
                    Self { interior, parameters, data, cost }
                }
                NodeType::Variable(v) => {
                    let parameters = v.parameter;
                    let data = !parameters;
                    let cost = 1;
                    Self { interior, parameters, data, cost }
                }
            }
        }
    }
    fn get_children(&self) -> Vec<*const Node> {
        match &self.interior {
            NodeType::Constant(_) => vec![],
            NodeType::Collection(c) => match c {
                Collection::Array(a) => a.elements.clone(),
                Collection::Struct(s) => s.elements.values().map(|v| v.clone()).collect(),
                Collection::Tuple(a) => a.elements.clone(),
            },
            NodeType::Variable(_) => vec![],
            NodeType::Unary(u) => vec![u.argument.clone()],
            NodeType::Binary(b) => vec![b.left.clone(), b.right.clone()],
        }
    }
}

impl PartialEq<f64> for Node {
    fn eq(&self, other: &f64) -> bool {
        if let NodeType::Constant(c) = &self.interior {
            c == other
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct Function {
    name: String,
    arguments: Vec<*const Node>,
    result: *const Node,
}

#[derive(Debug)]
pub struct Graph {
    vertices: HashSet<Box<Node>>,
    functions: HashMap<String, Function>,
}

impl Graph {
    pub fn new() -> Self {
        Self { vertices: HashSet::new(), functions: HashMap::new() }
    }

    pub(crate) fn new_collection(&mut self, collection: Collection) -> *const Node {
        self.insert(Node::new(NodeType::Collection(collection)))
    }

    pub(crate) fn new_constant(&mut self, value: f64) -> *const Node {
        self.insert(Node::new(NodeType::Constant(Constant { value })))
    }

    pub(crate) fn new_variable(&mut self, name: String, parameter: bool) -> *const Node {
        let var = Node::new(NodeType::Variable(Variable { name: name.clone(), parameter, index: 0 }));
        self.insert(var)
    }

    pub(crate) fn new_unary(&mut self, operand: UnaryOp, argument: *const Node) -> *const Node {
        Unary::new(self, operand, argument)
    }

    pub(crate) fn new_binary(&mut self, operand: BinaryOp, left: *const Node, right: *const Node) -> *const Node {
        Binary::new(self, operand, left, right)
    }

    pub fn insert(&mut self, node: Node) -> *const Node {
        let b = Box::new(node);
        if let Some(existing) = self.vertices.get(&b) {
            &**existing as *const Node
        } else {
            let ptr: *const Node = &*b;
            self.vertices.insert(b);
            ptr
        }
    }

    pub fn splice(&mut self, function: &Function, inputs: Vec<*const Node>) -> *const Node {
        function.validate_inputs(&inputs);
        let value = function.result;

        let mut map = HashMap::new();
        for i in 0..inputs.len() {
            map.insert(function.arguments[i].clone(), inputs[i].clone());
        }
        let order = function.order(&self);

        for node in order {
            let spliced_node = match &unsafe { &*node }.interior {
                NodeType::Binary(b) => self.new_binary(b.operation.clone(), map.get(&b.left).unwrap().clone(), map.get(&b.right).unwrap().clone()),
                NodeType::Collection(c) => match c {
                    Collection::Array(a) => {
                        let elements = a.elements.iter().map(|e| map.get(e).unwrap().clone()).collect();
                        self.new_collection(Collection::Array(Array { elements }))
                    }
                    Collection::Struct(s) => {
                        let name_order = s.name_order.clone();
                        let mut elements = HashMap::new();
                        name_order.iter().for_each(|name| {
                            let element = map.get(s.elements.get(name).unwrap()).unwrap().clone();
                            elements.insert(name.clone(), element);
                        });
                        self.new_collection(Collection::Struct(Struct { name: s.name.clone(), name_order, elements }))
                    }
                    Collection::Tuple(t) => {
                        let elements = t.elements.iter().map(|e| map.get(e).unwrap().clone()).collect();
                        self.new_collection(Collection::Tuple(Tuple { elements }))
                    }
                },
                NodeType::Constant(c) => self.new_constant(c.value),
                NodeType::Unary(u) => self.new_unary(u.operation.clone(), map.get(&u.argument).unwrap().clone()),
                NodeType::Variable(_) => map.get(&node).unwrap().clone(),
            };
            map.insert(node, spliced_node);
        }
        *map.get(&value).unwrap()
    }
    //
    // pub fn splice(&mut self, other: &Graph, inputs: Vec<*const Node>) -> *const Node {
    //     if inputs.len() != other.arguments.len() {
    //         panic!("could not splice graphs; input sizes do not match (self: {}, other: {}", inputs.len(), other.arguments.len());
    //     }
    //     let value = match &other.value {
    //         Some(v) => v,
    //         None => {
    //             panic!("attempted to splice a graph with no value");
    //         }
    //     };
    //
    //     let mut map = HashMap::new();
    //     for i in 0..inputs.len() {
    //         map.insert(other.arguments[i].clone(), inputs[i].clone());
    //     }
    //     let order = other.order();
    //
    //     for node in order {
    //         let spliced_node = match &node.interior {
    //             NodeType::Binary(b) => self.new_binary(b.operation.clone(), map.get(&b.left).unwrap().clone(), map.get(&b.right).unwrap().clone()),
    //             NodeType::Collection(c) => match c {
    //                 Collection::Array(a) => {
    //                     let elements = a.elements.iter().map(|e| map.get(e).unwrap().clone()).collect();
    //                     self.new_collection(Collection::Array(Array { elements }))
    //                 }
    //                 Collection::Struct(s) => {
    //                     let name_order = s.name_order.clone();
    //                     let mut elements = HashMap::new();
    //                     name_order.iter().for_each(|name| {
    //                         let element = map.get(s.elements.get(name).unwrap()).unwrap().clone();
    //                         elements.insert(name.clone(), element);
    //                     });
    //                     self.new_collection(Collection::Struct(Struct { name_order, elements }))
    //                 }
    //             },
    //             NodeType::Constant(c) => self.new_constant(c.value),
    //             NodeType::Unary(u) => self.new_unary(u.operation.clone(), map.get(&u.argument).unwrap().clone()),
    //             NodeType::Variable(_) => map.get(&node).unwrap().clone(),
    //         };
    //         map.insert(node, spliced_node);
    //     }
    //     map.get(value).unwrap().clone()
    // }

    pub fn differentiate(&mut self, node: *const Node, variable: &Variable) -> *const Node {
        match &unsafe { &*node }.interior {
            NodeType::Binary(b) => b.differentiate(self, variable),
            NodeType::Collection(_) => panic!("attempted to differentiate a collection"),
            NodeType::Constant(_) => self.new_constant(0.0),
            NodeType::Unary(u) => u.differentiate(self, variable),
            NodeType::Variable(v) => self.new_constant((v == variable) as u64 as f64),
        }
    }
}

impl Function {
    fn validate_inputs(&self, inputs: &Vec<*const Node>) {
        if inputs.len() != self.arguments.len() {
            panic!("function `{}` takes `{}` arguments, but was given `{}`", self.name, self.arguments.len(), inputs.len());
        }
    }

    pub fn compute_derivatives(&self, graph: &mut Graph) {
        let value = self.result;

        let variables = self
            .arguments
            .iter()
            .map(|argument| match &unsafe { &**argument }.interior {
                NodeType::Variable(v) => v,
                _ => {
                    panic!("unable to differentiate function `{}` with respect to non-variable argument", self.name);
                }
            })
            .collect::<Vec<_>>();

        let mut gradient = Vec::new();
        let mut hessian = Vec::new();

        for i in 0..variables.len() {
            let gradient_i = graph.differentiate(value, variables[i]);
            for j in i..variables.len() {
                hessian.push(graph.differentiate(gradient_i, variables[j]));
            }
            gradient.push(gradient_i);
        }

        let gradient_array = graph.new_collection(Collection::Array(collection::Array { elements: gradient }));
        let value_gradient_result = graph.new_collection(Collection::Tuple(collection::Tuple { elements: vec![value, gradient_array] }));
        let gradient_name = format!("{}_gradient", self.name);
        let gradient_function = Function { name: gradient_name.clone(), arguments: self.arguments.clone(), result: value_gradient_result };
        graph.functions.insert(gradient_name, gradient_function);

        let hessian_array = graph.new_collection(Collection::Array(collection::Array { elements: hessian }));
        let value_gradient_hessian_result = graph.new_collection(Collection::Tuple(collection::Tuple { elements: vec![value, gradient_array, hessian_array] }));
        let hessian_name = format!("{}_gradient_hessian", self.name);
        let gradient_hessian_function = Function { name: hessian_name.clone(), arguments: self.arguments.clone(), result: value_gradient_hessian_result };
        graph.functions.insert(hessian_name, gradient_hessian_function);
    }

    pub fn order(&self, graph: &Graph) -> Vec<*const Node> {
        let mut visited = HashSet::new();
        let mut sorted = Vec::new();

        fn dfs(graph: &Graph, node: &*const Node, visited: &mut HashSet<*const Node>, sorted: &mut Vec<*const Node>) {
            if visited.contains(node) {
                return;
            }
            visited.insert(node.clone());
            for child in &unsafe { &**node }.get_children() {
                dfs(graph, child, visited, sorted);
            }
            sorted.push(node.clone());
        }
        dfs(graph, &self.result, &mut visited, &mut sorted);
        sorted
    }
}
