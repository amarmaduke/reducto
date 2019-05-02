use std::collections::{HashMap};

use crate::tree::Tree;
use crate::strategy::Strategy;

#[derive(Debug, Clone)]
struct Environment {
    map: HashMap<usize, (Tree, Box<Environment>)>
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            map: HashMap::new()
        }
    }
}

#[derive(Debug, Clone)]
enum Frame {
    Closure(Tree, Environment),
    Argument(Tree, Environment)
}

type Continuations = Vec<Frame>;

#[derive(Debug, Clone)]
pub struct Machine {
    fresh : usize,
    code : (Tree, Option<Environment>),
    env : Environment,
    cc : Continuations,
    done : bool
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            fresh: 0,
            code: (Tree::Var(0), None),
            env: Environment::new(),
            cc: vec![],
            done: false
        }
    }

    fn transition(machine : Machine) -> Machine {
        let mut fresh = machine.fresh;
        let code = machine.code;
        let env = machine.env;
        let mut cc = machine.cc;

        match code {
            (Tree::App(left, right), _) => {
                cc.push(Frame::Argument(*right, env.clone()));
                Machine {
                    fresh,
                    code: (*left, None),
                    env,
                    cc,
                    done: false
                }
            },
            (Tree::Abs(id, body), None) => {
                Machine {
                    fresh,
                    code: (Tree::Abs(id, body), Some(env.clone())),
                    env,
                    cc,
                    done: false
                }
            },
            (Tree::Abs(id, mut body), Some(e1)) => {
                if let Some(k) = cc.pop() {
                    match k {
                        Frame::Closure(t, mut e2) => {
                            if let Tree::Abs(x, mut b) = t {
                                if e2.map.contains_key(&x) {
                                    fresh += 1;
                                    b.rename(x, fresh);
                                    body.rename(x, fresh);
                                    e2.map.insert(fresh, (Tree::Abs(id, body), Box::new(e1)));
                                } else {
                                    e2.map.insert(x, (Tree::Abs(id, body), Box::new(e1)));
                                }
                                Machine {
                                    fresh,
                                    code: (*b, None),
                                    env: e2,
                                    cc: cc,
                                    done: false
                                }
                            } else {
                                panic!("Impossible machine state.");
                            }
                        },
                        Frame::Argument(m, e2) => {
                            cc.push(Frame::Closure(Tree::Abs(id, body), e1));
                            Machine {
                                fresh,
                                code: (m, None),
                                env: e2,
                                cc,
                                done: false
                            }
                        }
                    }
                } else {
                    Machine {
                        fresh,
                        code: (Tree::Abs(id, body), Some(e1.clone())),
                        env: e1,
                        cc,
                        done: true
                    }
                }
            },
            (Tree::Var(v), _) => {
                let e2 = env.map.get(&v).expect("Impossible machine state.").clone();
                Machine {
                    fresh,
                    code: (e2.0, Some(*e2.1)),
                    env,
                    cc,
                    done: false
                }
            }
        }
    }

    fn reduce(machine : Machine) -> Machine {
        let mut result = machine;
        while !result.done {
            result = Machine::transition(result);
        }
        result
    }
}

impl Strategy for Machine {
    fn build(&mut self, tree : &Tree) {
        self.fresh = Tree::find_largest_var(tree);
        self.code = (tree.clone(), None);
    }

    fn reduce(&mut self) -> Option<u64> {
        *self = Machine::reduce(self.clone());
        //println!("{:?}", self);
        self.code.0.convert()
    }

    fn name(&self) -> String {
        String::from("cek machine")
    }
}
