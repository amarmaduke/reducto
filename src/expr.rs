use crate::tree::Tree;

#[derive(Debug)]
pub enum ArithBinaryOp {
    Add,
    Mul,
}

impl ArithBinaryOp {
    fn gen() -> ArithBinaryOp {
        let choice = rand::random::<usize>() % 100;
        use ArithBinaryOp::*;
        match choice {
            0 ... 100 => Add,
            _ => Mul,
        }
    }
}

#[derive(Debug)]
pub enum ArithExpr {
    Numeral(u64),
    Op(ArithBinaryOp, Box<ArithExpr>, Box<ArithExpr>)
}

impl ArithExpr {
    pub fn eval(&self) -> u64 {
        use ArithExpr::*;
        use ArithBinaryOp::*;
        match self {
            Numeral(x) => *x,
            Op(Add, left, right) => left.eval() + right.eval(),
            Op(Mul, left, right) => left.eval() * right.eval(),
        }
    }

    pub fn gen(depth : usize) -> ArithExpr {
        use ArithExpr::*;
        if depth == 0 {
            Numeral(rand::random::<u64>() % 2)
        } else {
            let left = ArithExpr::gen(depth - 1);
            let right = ArithExpr::gen(depth - 1);
            let op = ArithBinaryOp::gen();
            Op(op, Box::new(left), Box::new(right))
        }
    }

    pub fn gen_simple() -> ArithExpr {
        use ArithExpr::*;
        Op(ArithBinaryOp::Add,
            Box::new(Numeral(1)),
            Box::new(Numeral(0)));
        Numeral(0)
    }

    pub fn elab(&self, id : &mut usize) -> Tree {
        use ArithExpr::*;
        use ArithBinaryOp::*;
        match self {
            Numeral(n) => {
                let f = *id + 1;
                let x = *id + 2;
                *id += 2;
                let mut tree = Tree::Var(x);
                for _ in 0..*n {
                    tree = Tree::App(Box::new(Tree::Var(f)), Box::new(tree));
                }
                tree = Tree::Abs(x, Box::new(tree));
                tree = Tree::Abs(f, Box::new(tree));
                tree
            },
            Op(Add, left, right) => {
                let m = *id + 1;
                let n = *id + 2;
                let f = *id + 3;
                let x = *id + 4;
                *id += 4;
                let add = Tree::Abs(m,
                    Box::new(Tree::Abs(n,
                    Box::new(Tree::Abs(f,
                    Box::new(Tree::Abs(x,
                    Box::new(Tree::App(
                        Box::new(Tree::App(
                            Box::new(Tree::Var(m)),
                            Box::new(Tree::Var(f))
                        )),
                        Box::new(Tree::App(
                            Box::new(Tree::App(
                                Box::new(Tree::Var(n)),
                                Box::new(Tree::Var(f))
                            )),
                            Box::new(Tree::Var(x))
                        ))
                    )))))))));
                Tree::App(
                    Box::new(Tree::App(
                        Box::new(add),
                        Box::new(left.elab(id))
                    )),
                    Box::new(right.elab(id))
                )
            }
            Op(Mul, left, right) => {
                let m = *id + 1;
                let n = *id + 2;
                let f = *id + 3;
                *id += 3;
                let mul = Tree::Abs(m,
                    Box::new(Tree::Abs(n,
                    Box::new(Tree::Abs(f,
                    Box::new(Tree::App(
                        Box::new(Tree::Var(m)),
                        Box::new(Tree::App(
                            Box::new(Tree::Var(n)),
                            Box::new(Tree::Var(f))
                        ))
                    )))))));
                Tree::App(
                    Box::new(Tree::App(
                        Box::new(mul),
                        Box::new(left.elab(id))
                    )),
                    Box::new(right.elab(id))
                )
            }
        }
    }
}

#[derive(Debug)]
pub enum VariableExpr {
    Numeral(u64),
    Var1,
    Var2,
    Op(ArithBinaryOp, Box<VariableExpr>, Box<VariableExpr>)
}

impl VariableExpr {
    fn eval(&self, v1 : u64, v2 : u64) -> u64 {
        use VariableExpr::*;
        use ArithBinaryOp::*;
        match self {
            Numeral(x) => *x,
            Var1 => v1,
            Var2 => v2,
            Op(Add, left, right) => left.eval(v1, v2) + right.eval(v1, v2),
            Op(Mul, left, right) => left.eval(v1, v2) * right.eval(v1, v2),
        }
    }

    fn gen(depth : usize, var_count : usize) -> VariableExpr {
        use VariableExpr::*;
        if depth == 0 {
            let choice : usize = if var_count > 1 {
                rand::random::<usize>() % 3
            } else if var_count == 1 {
                rand::random::<usize>() % 2
            } else {
                0
            };
            match choice {
                0 => Numeral(rand::random::<u64>() % 2),
                1 => Var1,
                _ => Var2
            }
        } else {
            let left = VariableExpr::gen(depth - 1, var_count);
            let right = VariableExpr::gen(depth - 1, var_count);
            let op = ArithBinaryOp::gen();
            Op(op, Box::new(left), Box::new(right))
        }
    }

    fn elab_helper(&self, u : usize, v : usize, id : &mut usize) -> Tree {
        use VariableExpr::*;
        use ArithBinaryOp::*;
        match self {
            Numeral(n) => {
                let f = *id + 1;
                let x = *id + 2;
                *id += 2;
                let mut tree = Tree::Var(x);
                for _ in 0..*n {
                    tree = Tree::App(Box::new(Tree::Var(f)), Box::new(tree));
                }
                tree = Tree::Abs(x, Box::new(tree));
                tree = Tree::Abs(f, Box::new(tree));
                tree
            },
            Var1 => Tree::Var(u),
            Var2 => Tree::Var(v),
            Op(Add, left, right) => {
                let m = *id + 1;
                let n = *id + 2;
                let f = *id + 3;
                let x = *id + 4;
                *id += 4;
                let add = Tree::Abs(m,
                    Box::new(Tree::Abs(n,
                    Box::new(Tree::Abs(f,
                    Box::new(Tree::Abs(x,
                    Box::new(Tree::App(
                        Box::new(Tree::App(
                            Box::new(Tree::Var(m)),
                            Box::new(Tree::Var(f))
                        )),
                        Box::new(Tree::App(
                            Box::new(Tree::App(
                                Box::new(Tree::Var(n)),
                                Box::new(Tree::Var(f))
                            )),
                            Box::new(Tree::Var(x))
                        ))
                    )))))))));
                Tree::App(
                    Box::new(Tree::App(
                        Box::new(add),
                        Box::new(left.elab_helper(u, v, id))
                    )),
                    Box::new(right.elab_helper(u, v, id))
                )
            }
            Op(Mul, left, right) => {
                let m = *id + 1;
                let n = *id + 2;
                let f = *id + 3;
                *id += 3;
                let mul = Tree::Abs(m,
                    Box::new(Tree::Abs(n,
                    Box::new(Tree::Abs(f,
                    Box::new(Tree::App(
                        Box::new(Tree::Var(m)),
                        Box::new(Tree::App(
                            Box::new(Tree::Var(n)),
                            Box::new(Tree::Var(f))
                        ))
                    )))))));
                Tree::App(
                    Box::new(Tree::App(
                        Box::new(mul),
                        Box::new(left.elab_helper(u, v, id))
                    )),
                    Box::new(right.elab_helper(u, v, id))
                )
            }
        }
    }

    fn elab(&self, var_count : usize, id : &mut usize) -> Tree {
        let u = *id + 1;
        let v = *id + 2;
        *id += var_count;
        let mut tree = self.elab_helper(u, v, id);
        if var_count == 1 {
            tree = Tree::Abs(u, Box::new(tree));
        } else if var_count == 2 {
            tree = Tree::Abs(u,
                Box::new(Tree::Abs(v, Box::new(tree))));
        }
        tree
    }
}

#[derive(Debug)]
pub enum ListMapSequence {
    MapSeq(Vec<VariableExpr>, Vec<ArithExpr>)
}

impl ListMapSequence {
    fn eval(&self) -> Vec<u64> {
        use ListMapSequence::*;
        match self {
            MapSeq(ops, list) => {
                let mut result : Vec<_> = list.iter().map(|x| x.eval()).collect();
                for op in ops.iter() {
                    result = result.iter().map(|x| op.eval(*x, 0)).collect();
                }
                result
            }
        }
    }

    fn gen(depth : usize, len : usize) -> ListMapSequence {
        use ListMapSequence::*;
        let mut ops = vec![];
        let mut exprs = vec![];
        for _ in 0..depth {
            ops.push(VariableExpr::gen(1 + depth/2, 1));
        }
        for _ in 0..len {
            exprs.push(ArithExpr::gen(depth));
        }
        MapSeq(ops, exprs)
    }

    fn elab_map(&self, id : &mut usize) -> Tree {
        let f = *id + 1;
        let arg = *id + 2;
        let cons = *id + 3;
        let nil = *id + 4;
        let x = *id + 5;
        *id += 5;
        Tree::Abs(f,
            Box::new(Tree::Abs(arg,
            Box::new(Tree::Abs(cons,
            Box::new(Tree::Abs(nil,
            Box::new(Tree::App(
                Box::new(Tree::App(
                    Box::new(Tree::Var(arg)),
                    Box::new(Tree::Abs(x,
                    Box::new(Tree::App(
                        Box::new(Tree::Var(cons)),
                        Box::new(Tree::App(
                            Box::new(Tree::Var(f)),
                            Box::new(Tree::Var(x))
                        ))
                    ))))
                )),
                Box::new(Tree::Var(nil)),
            ))
        )))))))
    }

    fn elab_list(&self, id : &mut usize) -> Tree {
        let nil = *id + 1;
        let cons = *id + 2;
        *id += 2;
        let mut tree = Tree::Var(nil);
        let ListMapSequence::MapSeq(_, list) = self;
        for x in list.iter().rev() {
            tree = Tree::App(
                Box::new(Tree::App(
                    Box::new(Tree::Var(cons)),
                    Box::new(x.elab(id))
                )),
                Box::new(tree)
            );
        }
        tree = Tree::Abs(nil, Box::new(tree));
        tree = Tree::Abs(cons, Box::new(tree));
        tree
    }

    fn elab(&self, id : &mut usize) -> Tree {
        let ListMapSequence::MapSeq(maps, _) = self;
        let mut tree = self.elab_list(id);
        for m in maps.iter() {
            tree = Tree::App(
                Box::new(Tree::App(
                    Box::new(self.elab_map(id)),
                    Box::new(m.elab(1, id))
                )),
                Box::new(tree)
            );
        }
        tree
    }
}

#[derive(Debug)]
pub enum ListFold {
    Fold(VariableExpr, ArithExpr, ListMapSequence)
}

impl ListFold {
    pub fn eval(&self) -> u64 {
        use ListFold::*;
        match self {
            Fold(op, init, list) => {
                list.eval().iter()
                    .fold(init.eval(), |a, x| op.eval(*x, a))
            }
        }
    }

    pub fn gen(depth : usize, len : usize) -> ListFold {
        let op = VariableExpr::Op(ArithBinaryOp::Add,
            Box::new(VariableExpr::Var1),
            Box::new(VariableExpr::Var2));
        let init = ArithExpr::gen(depth);
        let seq = ListMapSequence::gen(depth, len);
        ListFold::Fold(op, init, seq)
    }

    pub fn gen_simple() -> ListFold {
        let op = VariableExpr::Op(ArithBinaryOp::Add,
            Box::new(VariableExpr::Var1),
            Box::new(VariableExpr::Var2));
        let init = ArithExpr::gen_simple();
        let seq = ListMapSequence::gen(0, 0);
        ListFold::Fold(op, init, seq)
    }

    fn elab_fold(&self, id : &mut usize) -> Tree {
        let f = *id + 1;
        let init = *id + 2;
        let arg = *id + 3;
        let x = *id + 4;
        *id += 4;
        Tree::Abs(f,
            Box::new(Tree::Abs(init,
            Box::new(Tree::Abs(arg,
            Box::new(Tree::App(
                Box::new(Tree::App(
                    Box::new(Tree::Var(arg)),
                    Box::new(Tree::Abs(x,
                        Box::new(Tree::App(
                            Box::new(Tree::Var(f)),
                            Box::new(Tree::Var(x))
                        ))))
                )),
                Box::new(Tree::Var(init))
        )))))))
    }

    pub fn elab(&self, id : &mut usize) -> Tree {
        let ListFold::Fold(op, init, list) = self;
        Tree::App(
            Box::new(Tree::App(
                Box::new(Tree::App(
                    Box::new(self.elab_fold(id)),
                    Box::new(op.elab(2, id))
                )),
                Box::new(init.elab(id))
            )),
            Box::new(list.elab(id))
        )
    }
}
