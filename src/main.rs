use std::time::Instant;

mod tree;
mod expr;
mod strategy;
mod normal;
mod cek;

use crate::tree::Tree;
use crate::expr::ListFold;
use crate::strategy::Strategy;
use crate::cek::Machine;

fn benchmark(strategies : &mut Vec<Box<Strategy>>, depth : usize, len : usize) {
    let mut averages : Vec<_> = strategies.iter().map(|_| 0.0).collect();
    let (sample, measure) = (3, 3);
    for _ in 0..sample {
        let mut id = 0;
        let expr = ListFold::gen(depth, len);
        let value = expr.eval();
        let tree = expr.elab(&mut id);
        //println!("expr: {:?}", expr);
        //println!("result: {}", value);
        for i in 0..strategies.len() {
            let ref mut strategy = strategies[i];
            strategy.build(&tree);
            let output = strategy.reduce();
            assert_eq!(value, output.expect("Invalid u64 value."));
            let now = Instant::now();
            for _ in 0..measure {
                strategy.build(&tree);
                strategy.reduce();
            }
            let time = now.elapsed();
            averages[i] += (time.as_secs() as f64
                + time.subsec_nanos() as f64 * 1e-9) / (measure as f64);
        }
    }
    for i in 0..strategies.len() {
        averages[i] /= sample as f64;
        println!("{}: {}s", strategies[i].name(), averages[i]);
    }
}

fn main() {
    let mut strategies : Vec<Box<Strategy>> = vec![];
    strategies.push(Box::new(Tree::Var(0)));
    strategies.push(Box::new(Machine::new()));
    benchmark(&mut strategies, 1, 3);
}
