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
use crate::expression::collection::{Array, Struct};
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
                    Self {
                        interior,
                        parameters,
                        data,
                        cost,
                    }
                }
                NodeType::Collection(_) => Self {
                    interior,
                    parameters: false,
                    data: false,
                    cost: 0,
                },
                NodeType::Constant(_) => Self {
                    interior,
                    parameters: false,
                    data: false,
                    cost: 1,
                },
                NodeType::Unary(u) => {
                    let parameters = (*u.argument).parameters;
                    let data = (*u.argument).data;
                    let cost = (*u.argument).cost + u.operation.cost();
                    Self {
                        interior,
                        parameters,
                        data,
                        cost,
                    }
                }
                NodeType::Variable(v) => {
                    let parameters = v.parameter;
                    let data = !parameters;
                    let cost = 1;
                    Self {
                        interior,
                        parameters,
                        data,
                        cost,
                    }
                }
            }
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
pub(crate) struct Graph {
    nodes: HashSet<Box<Node>>,
    pub(crate) value: Option<*const Node>,
    pub(crate) arguments: Vec<*const Node>,
    pub(crate) data: Vec<*const Node>,
    pub(crate) gradient: Vec<*const Node>,
    pub(crate) hessian: Vec<*const Node>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            value: None,
            arguments: Vec::new(),
            data: Vec::new(),
            gradient: Vec::new(),
            hessian: Vec::new(),
        }
    }

    pub(crate) fn new_collection(&mut self, collection: Collection) -> *const Node {
        self.insert(Node::new(NodeType::Collection(collection)))
    }

    pub(crate) fn new_constant(&mut self, value: f64) -> *const Node {
        self.insert(Node::new(NodeType::Constant(Constant { value })))
    }

    pub(crate) fn new_variable(&mut self, name: String, parameter: bool) -> *const Node {
        let var = Node::new(NodeType::Variable(Variable {
            name: name.clone(),
            parameter,
            index: 0,
        }));
        if let Some(_) = self.nodes.get(&var) {
            return self.insert(var);
        }
        if parameter {
            let var = Node::new(NodeType::Variable(Variable {
                name,
                parameter,
                index: self.arguments.len(),
            }));
            let result = self.insert(var);
            self.arguments.push(result);
            result
        } else {
            let var = Node::new(NodeType::Variable(Variable {
                name,
                parameter,
                index: self.data.len(),
            }));
            let result = self.insert(var);
            self.data.push(result);
            result
        }
    }

    pub(crate) fn new_unary(&mut self, operand: UnaryOp, argument: *const Node) -> *const Node {
        Unary::new(self, operand, argument)
    }

    pub(crate) fn new_binary(
        &mut self,
        operand: BinaryOp,
        left: *const Node,
        right: *const Node,
    ) -> *const Node {
        Binary::new(self, operand, left, right)
    }

    pub fn insert(&mut self, node: Node) -> *const Node {
        let b = Box::new(node);
        if let Some(existing) = self.nodes.get(&b) {
            &**existing as *const Node
        } else {
            let ptr: *const Node = &*b;
            self.nodes.insert(b);
            ptr
        }
    }

    pub fn splice(&mut self, other: &Graph, inputs: Vec<*const Node>) -> *const Node {
        if inputs.len() != other.arguments.len() {
            panic!(
                "could not splice graphs; input sizes do not match (self: {}, other: {}",
                inputs.len(),
                other.arguments.len()
            );
        }
        let mut map = HashMap::new();
        for i in 0..inputs.len() {
            map.insert(other.arguments[i], inputs[i]);
        }
        let order = other.order();

        for node in order {
            let spliced_node = match &unsafe { &*node }.interior {
                NodeType::Binary(b) => self.new_binary(
                    b.operation.clone(),
                    *map.get(&b.left).unwrap(),
                    *map.get(&b.right).unwrap(),
                ),
                NodeType::Collection(c) => match c {
                    Collection::Array(a) => {
                        let elements = a.elements.iter().map(|e| *map.get(e).unwrap()).collect();
                        self.new_collection(Collection::Array(Array { elements }))
                    }
                    Collection::Struct(s) => {
                        let name_order = s.name_order.clone();
                        let mut elements = HashMap::new();
                        name_order.iter().for_each(|name| {
                            let element = *map.get(s.elements.get(name).unwrap()).unwrap();
                            elements.insert(name.clone(), element);
                        });
                        self.new_collection(Collection::Struct(Struct {
                            name_order,
                            elements,
                        }))
                    }
                },
                NodeType::Constant(c) => self.new_constant(c.value),
                NodeType::Unary(u) => {
                    self.new_unary(u.operation.clone(), *map.get(&u.argument).unwrap())
                }
                NodeType::Variable(_) => *map.get(&node).unwrap(),
            };
            map.insert(node, spliced_node);
        }
        *map.get(&other.value.unwrap()).unwrap()
    }

    pub fn differentiate(&mut self, node: *const Node, variable: &Variable) -> *const Node {
        match &unsafe { &*node }.interior {
            NodeType::Binary(b) => b.differentiate(self, variable),
            NodeType::Collection(_) => panic!("attempted to differentiate a collection"),
            NodeType::Constant(_) => self.new_constant(0.0),
            NodeType::Unary(u) => u.differentiate(self, variable),
            NodeType::Variable(v) => self.new_constant((v == variable) as u64 as f64),
        }
    }

    pub fn compute_gradient(&mut self) {
        if self.value.is_none() {
            panic!("attempted to differentiate a graph with no value");
        }
        if let NodeType::Collection(_) = unsafe { &*self.value.unwrap() }.interior {
            panic!("attempted to differentiate a collection");
        }
        for node in self.arguments.clone() {
            let argument = if let NodeType::Variable(v) = &unsafe { &*node }.interior {
                v
            } else {
                panic!(
                    "attempted to differentiate with respect to a non-variable node during gradient computation"
                )
            };
            let derivative = self.differentiate(self.value.unwrap(), argument);
            self.gradient.push(derivative);
        }
    }

    pub fn compute_hessian(&mut self) {
        let mut i = 0;
        let size = self.gradient.len();
        for component in self.gradient.clone() {
            for index in i..size {
                let node = self.arguments[index];
                let argument = if let NodeType::Variable(v) = &unsafe { &*node }.interior {
                    v
                } else {
                    panic!(
                        "attempted to differentiate with respect to a non-variable node during hessian computation"
                    )
                };
                let hess = self.differentiate(component, argument);
                self.hessian.push(hess);
            }
            i += 1;
        }
    }

    fn get_children(&self, node: *const Node) -> Vec<*const Node> {
        match &unsafe { &*node }.interior {
            NodeType::Constant(_) => vec![],
            NodeType::Collection(c) => match c {
                Collection::Array(a) => a.elements.clone(),
                Collection::Struct(s) => s.elements.values().map(|v| *v).collect(),
            },
            NodeType::Variable(_) => vec![],
            NodeType::Unary(u) => vec![u.argument],
            NodeType::Binary(b) => vec![b.left, b.right],
        }
    }

    pub fn order(&self) -> Vec<*const Node> {
        let mut visited = HashSet::new();
        let mut sorted = Vec::new();

        fn dfs(
            graph: &Graph,
            node: *const Node,
            visited: &mut HashSet<*const Node>,
            sorted: &mut Vec<*const Node>,
        ) {
            if visited.contains(&node) {
                return;
            }
            visited.insert(node);
            for &child in &graph.get_children(node) {
                dfs(graph, child, visited, sorted);
            }
            sorted.push(node);
        }

        if let Some(val) = self.value {
            dfs(self, val, &mut visited, &mut sorted)
        }

        for &grad in &self.gradient {
            dfs(self, grad, &mut visited, &mut sorted);
        }

        for &hess in &self.hessian {
            dfs(self, hess, &mut visited, &mut sorted);
        }
        sorted
    }
}
