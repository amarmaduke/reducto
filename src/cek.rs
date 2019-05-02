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
    code : (Tree, Environment),
    env : Environment,
    cc : Continuations,
    done : bool
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            code: (Tree::Var(0), Environment::new()),
            env: Environment::new(),
            cc: vec![],
            done: false
        }
    }

    fn transition(machine : Machine) -> Machine {
        let code = machine.code;
        let env = machine.env;
        let mut cc = machine.cc;

        match code {
            (Tree::App(left, right), e1) => {
                cc.push(Frame::Argument(*right, env.clone()));
                Machine {
                    code: (*left, e1),
                    env: env,
                    cc: cc,
                    done: false
                }
            },
            (Tree::Abs(id, body), e1) => {
                if let Some(k) = cc.pop() {
                    match k {
                        Frame::Closure(t, mut e2) => {
                            if let Tree::Abs(id, b) = t {
                                e2.map.insert(id, (Tree::Abs(id, body), Box::new(e1)));
                                Machine {
                                    code: (*b, Environment::new()),
                                    env: e2,
                                    cc: cc,
                                    done: false
                                }
                            } else {
                                println!("{:?}", t);
                                panic!("Impossible machine state.");
                            }
                        },
                        Frame::Argument(t, e2) => {
                            cc.push(Frame::Closure(Tree::Abs(id, body), e1));
                            Machine {
                                code: (t, Environment::new()),
                                env: e2,
                                cc: cc,
                                done: false
                            }
                        }
                    }
                } else {
                    Machine {
                        code: (Tree::Abs(id, body), e1.clone()),
                        env: e1,
                        cc: cc,
                        done: true
                    }
                }
            },
            (Tree::Var(v), _) => {
                let e2 = env.map.get(&v).expect("Impossible machine state.").clone();
                Machine {
                    code: (e2.0, *e2.1),
                    env: env,
                    cc: cc,
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
        self.code = (tree.clone(), Environment::new());
    }

    fn reduce(&mut self) -> Option<u64> {
        *self = Machine::reduce(self.clone());
        self.code.0.convert()
    }

    fn name(&self) -> String {
        String::from("cek machine")
    }
}
