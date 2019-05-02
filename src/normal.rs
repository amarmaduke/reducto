use std::collections::{HashMap};

use crate::tree::Tree;
use crate::strategy::Strategy;

impl Tree {
    fn shift(tree : Tree, place : isize, cutoff : isize) -> Tree {
        match tree {
            Tree::Var(index) => {
                let i = index as isize;
                if i < cutoff {
                    Tree::Var(index)
                } else {
                    Tree::Var((i + place) as usize)
                }
            },
            Tree::Abs(id, body) => {
                Tree::Abs(id, Box::new(Tree::shift(*body, place, cutoff + 1)))
            },
            Tree::App(left, right) => {
                Tree::App(
                    Box::new(Tree::shift(*left, place, cutoff)),
                    Box::new(Tree::shift(*right, place, cutoff)))
            }
        }
    }

    fn is_normal(&self) -> bool {
        let mut result = true;
        match self {
            Tree::App(left, right) => {
                if let Tree::Abs(_, _) = **left {
                    result &= false;
                } else {
                    result &= left.is_normal();
                    result &= right.is_normal();
                }
            },
            Tree::Abs(_, expr) => {
                result &= expr.is_normal();
            }
            Tree::Var(_) => { }
        }
        result
    }

    fn substitute(tree : Tree, argument : Tree, depth : usize) -> Tree {
        match tree {
            Tree::Var(index) if index == depth => {
                argument
            },
            Tree::Var(index) => Tree::Var(index),
            Tree::Abs(id, expr)
                => Tree::Abs(id, Box::new(Tree::substitute(*expr, Tree::shift(argument, 1, 0), depth + 1))),
            Tree::App(left, right) =>
                Tree::App(Box::new(Tree::substitute(*left, argument.clone(), depth)),
                    Box::new(Tree::substitute(*right, argument, depth)))
        }
    }

    fn reduction_step(tree : Tree) -> Tree {
        match tree {
            Tree::Var(index) => Tree::Var(index),
            Tree::Abs(id, expr) => Tree::Abs(id, Box::new(Tree::reduction_step(*expr))),
            Tree::App(left, right) => {
                if let Tree::Abs(_, expr) = *left {
                    Tree::shift(Tree::substitute(*expr, Tree::shift(*right, 1, 0), 0), -1, 0)
                } else {
                    Tree::App(Box::new(Tree::reduction_step(*left)),
                        Box::new(Tree::reduction_step(*right)))
                }
            }
        }
    }

    pub fn reduce(tree : Tree) -> Tree {
        let mut result = tree;
        loop {
            result = Tree::reduction_step(result);
            //println!("{}", result.to_indexed_string());
            if result.is_normal() { break; }
        }
        result
    }

    pub fn fix_indices(mut tree : Tree) -> Tree {
        let mut map = HashMap::new();
        Tree::fix_indices_helper(&mut tree, &mut map, 0);
        tree
    }

    fn fix_indices_helper(tree : &mut Tree, map : &mut HashMap<usize, Vec<usize>>, depth : usize) {
        match tree {
            Tree::Var(ref mut index) => {
                let id = *index;
                *index = 100;
                if let Some(stack) = map.get(&id) {
                    if let Some(relative_depth) = stack.last() {
                        *index = depth - relative_depth - 1;
                    }
                }
            },
            Tree::Abs(id, ref mut body) => {
                {
                    let stack = map.entry(*id).or_insert(vec![]);
                    stack.push(depth);
                }
                Tree::fix_indices_helper(body, map, depth + 1);
                let stack = map.entry(*id).or_insert(vec![]);
                stack.pop();
            },
            Tree::App(ref mut left, ref mut right) => {
                Tree::fix_indices_helper(left, map, depth);
                Tree::fix_indices_helper(right, map, depth);
            }
        }
    }
}

impl Strategy for Tree {
    fn build(&mut self, tree : &Tree) {
        *self = Tree::fix_indices(tree.clone());
        //println!("{:?}", self.to_indexed_string());
    }

    fn reduce(&mut self) -> Option<u64> {
        *self = Tree::reduce(self.clone());
        self.convert()
    }

    fn name(&self) -> String {
        String::from("de bruijn indices normal order")
    }
}
