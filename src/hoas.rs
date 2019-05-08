use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::{Display, Error, Formatter};
use std::collections::HashMap;

use crate::tree::Tree;
use crate::strategy::Strategy;

// inspiration for this taken from:
// https://stackoverflow.com/questions/51182640/is-it-possible-to-represent-higher-order-abstract-syntax-in-rust#

pub trait HoasFn {
    fn apply(&self, t: Hoas) -> Hoas;
    fn clone_box(&self) -> Box<dyn HoasFn>;
}

impl<F> HoasFn for F
where F: 'static + Clone + FnOnce(Hoas) -> Hoas,
{
    fn apply(&self, t: Hoas) -> Hoas {
        (self.clone())(t)
    }

    fn clone_box(&self) -> Box<dyn HoasFn> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn HoasFn> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Clone)]
pub enum Hoas {
    Var(isize),
    Abs(Box<dyn HoasFn>),
    App(Box<Hoas>, Box<Hoas>)
}

impl Hoas {
    fn app(t1: Self, t2: Self) -> Self {
        Hoas::App(Box::new(t1), Box::new(t2))
    }

    fn lam(t : impl 'static + Clone + FnOnce(Hoas) -> Hoas) -> Self {
        Hoas::Abs(Box::new(t))
    }

    pub fn new() -> Self {
        Self::lam(|x| Self::app(x.clone(), x))
    }

    fn step(term : Self) -> Self {
        use Hoas::*;
        match term {
            Var(_) => term,
            Abs(body) => {
                Hoas::lam(move |x| Hoas::step(body.apply(x)))
            },
            App(f, arg) => match Hoas::step(*f) {
                Abs(body) => body.apply(*arg),
                t => Hoas::app(t, Hoas::step(*arg))
            }
        }
    }

    fn convert(tree : Tree, map : Rc<RefCell<HashMap<usize, Hoas>>>) -> Hoas {
        match tree {
            Tree::Var(x) => {
                if let Some(term) = map.borrow().get(&x) {
                    term.clone()
                } else {
                    Hoas::Var(x as isize)
                }
            },
            Tree::Abs(id, body) => {
                Hoas::lam(move |x| {
                    map.borrow_mut().insert(id, x.clone());
                    Hoas::convert(*body, map)
                })
            },
            Tree::App(f, arg) => {
                Hoas::app(
                    Hoas::convert(*f, map.clone()),
                    Hoas::convert(*arg, map))
            }
        }
    }
}

impl Display for Hoas {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        struct Helper<'a>(usize, &'a Hoas);
        impl<'a> Display for Helper<'a> {
            fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
                use Hoas::*;
                match self {
                    Helper(_, Var(i)) => write!(fmt, "x{}", i),
                    Helper(lvl, Abs(body)) => write!(fmt, "λx{}. {}", *lvl,
                        Helper(*lvl + 1, &body.apply(Var(*lvl as isize)))),
                    Helper(lvl, App(f, arg)) => write!(fmt, "({} {})",
                        Helper(*lvl, f), Helper(*lvl, arg))
                }
            }
        }
        write!(fmt, "{}", Helper(0, self))
    }
}

impl Strategy for Hoas {
    fn build(&mut self, tree : &Tree) {
        let map = Rc::new(RefCell::new(HashMap::new()));
        let term = Hoas::convert(tree.clone(), map);
        *self = term;
    }

    fn reduce(&mut self) -> Option<u64> {
        use Hoas::*;
        *self = Hoas::step(self.clone());
        // FIXME: Somehow printing modifies self?
        //println!("{}", self);
        let mut result = None;
        if let Abs(body) = self {
            let next = body.apply(Var(-1));
            if let Abs(body) = next {
                let mut next = body.apply(Var(-1));
                let mut counter = 0i64;
                loop {
                    if let App(ref left, ref right) = next {
                        if let Var(ref i) = **left {
                            counter += *i as i64;
                        } else { break; }
                        next = *right.clone();
                    } else { break; }
                }
                result = Some((-counter) as u64);
            }
        }
        result
    }

    fn name(&self) -> String {
        String::from("hoas")
    }
}
