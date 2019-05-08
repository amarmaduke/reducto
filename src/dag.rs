use std::collections::{HashMap};

use crate::tree::Tree;
use crate::strategy::Strategy;

fn remove_item(item : usize, vec : &mut Vec<usize>) {
    let mut index = 0;
    let mut found = false;
    for i in 0..vec.len() {
        if vec[i] == item {
            index = i;
            found = true;
        }
    }
    if found {
        vec.remove(index);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Kind {
    Var,
    Abs,
    App
}

#[derive(Debug)]
struct Node {
    kind : Kind,
    parents : Vec<usize>,
    left : usize,
    right : usize
}

impl Node {
    fn var(parents : Vec<usize>) -> Node {
        Node {
            kind: Kind::Var,
            parents,
            left: 0,
            right: 0
        }
    }

    fn abs(left : usize, right : usize, parents : Vec<usize>) -> Node {
        Node {
            kind: Kind::Abs,
            parents,
            left,
            right
        }
    }

    fn app(left : usize, right : usize, parents : Vec<usize>) -> Node {
        Node {
            kind: Kind::App,
            parents,
            left,
            right
        }
    }
}

#[derive(Debug)]
pub struct Dag {
    id : usize,
    nodes : HashMap<usize, Node>
}

impl Dag {
    pub fn new() -> Dag {
        Dag { id: 0, nodes: HashMap::new() }
    }

    fn insert(&mut self, node : Node) -> usize {
        self.id += 1;
        self.nodes.insert(self.id, node);
        self.id
    }

    fn reserve(dag : &mut Dag, mut continuation : impl FnMut(&mut Dag, usize) -> Node) -> usize {
        dag.id += 1;
        let reserved = dag.id;
        let node = continuation(dag, reserved);
        dag.nodes.insert(reserved, node);
        reserved
    }

    fn remove(&mut self, id : usize) -> Option<Node> {
        self.nodes.remove(&id)
    }

    fn get_mut(&mut self, id : usize) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    pub fn from(tree : &Tree) -> Dag {
        let mut dag = Dag::new();
        let mut map = HashMap::new();
        dag.from_helper(tree, vec![], &mut map);
        dag.fix_abs_refs(&mut map);
        dag
    }

    fn fix_abs_refs(&mut self, map : &mut HashMap<usize, usize>) {
        for (_, node) in self.nodes.iter_mut() {
            if node.kind == Kind::Abs {
                if let Some(v) = map.get(&node.left) {
                    node.left = *v;
                } else {
                    node.left = 0;
                }
            }
        }
    }

    fn from_helper(&mut self, tree : &Tree, parents : Vec<usize>, map : &mut HashMap<usize, usize>) -> usize {
        match tree {
            Tree::Var(id) => {
                if let Some(v) = map.get(id) {
                    let node = self.get_mut(*v)
                        .expect("impossible");
                    node.parents.extend(parents);
                    *v
                } else {
                    let v = self.insert(Node::var(parents));
                    map.insert(*id, v);
                    v
                }
            },
            Tree::Abs(id, body) => {
                Dag::reserve(self, |dag, abs_id| {
                    let body_id = dag.from_helper(body, vec![abs_id], map);
                    Node::abs(*id, body_id, parents.clone())
                })
            },
            Tree::App(left, right) => {
                Dag::reserve(self, |dag, app_id| {
                    let left_id = dag.from_helper(left, vec![app_id], map);
                    let right_id = dag.from_helper(right, vec![app_id], map);
                    Node::app(left_id, right_id, parents.clone())
                })
            }
        }
    }

    fn step(&mut self, id : usize) {
        let app = if let Some(app) = self.remove(id) { app } else { return; };
        let lam = if let Some(lam) = self.remove(app.left) { lam } else { return; };
        let varid = lam.left;

        // handle argument part
        let argid = app.right;
        if varid != 0 {
            let mut var = if let Some(var) = self.remove(varid) { var }
                else { return; };
            remove_item(id, &mut var.parents);
            remove_item(app.left, &mut var.parents);
            for i in var.parents.iter() {
                let node = if let Some(node) = self.get_mut(*i) { node }
                    else { return; };
                if node.left == varid {
                    node.left = argid;
                }
                if node.right == varid {
                    node.right = argid;
                }
            }
            if self.nodes.get(&argid).is_some() {
                let arg = if let Some(arg) = self.get_mut(argid) { arg }
                    else {return; }; 
                remove_item(id, &mut arg.parents);
                arg.parents.extend(var.parents);
            }
        } else {
            let arg = self.nodes.remove(&argid);
            if let Some(arg) = arg {
                let mut stack = vec![arg];
                while let Some(node) = stack.pop() {
                    let left = self.nodes.remove(&node.left);
                    let right = self.nodes.remove(&node.right);
                    if let Some(left) = left { stack.push(left); }
                    if let Some(right) = right { stack.push(right); }
                }
            }
        }

        // handle function part
        let bodyid = lam.right;
        for i in app.parents.iter() {
            let node = if let Some(node) = self.get_mut(*i) { node }
                else { return; };
            if node.left == id {
                node.left = bodyid;
            }
            if node.right == id {
                node.right = bodyid;
            }
        }
        
        if self.nodes.get(&bodyid).is_some() {
            let body = if let Some(body) = self.get_mut(bodyid) { body }
                else { return; };
            remove_item(app.left, &mut body.parents);
            body.parents.extend(app.parents);
        }
    }

    fn find_redex(&self) -> Option<usize> {
        for (id, node) in self.nodes.iter() {
            if node.kind == Kind::App {
                let left = self.nodes.get(&node.left);
                if let Some(left) = left {
                    if left.kind == Kind::Abs {
                        return Some(*id);
                    }
                }
            }
        }
        None
    }

    fn reduce(&mut self) {
        loop {
            let redex = self.find_redex();
            if let Some(redex) = redex {
                self.step(redex);
            } else { break; }
        }
    }
}

impl Strategy for Dag {
    fn build(&mut self, tree : &Tree) {
        *self = Dag::from(tree);
    }

    fn reduce(&mut self) -> Option<u64> {
        self.reduce();
        Some(0)
    }

    fn name(&self) -> String {
        String::from("dag")
    }
}
