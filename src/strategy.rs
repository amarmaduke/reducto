use crate::tree::Tree;

pub trait Strategy {
    fn build(&mut self, tree : &Tree);
    fn reduce(&mut self) -> Option<u64>;
    fn name(&self) -> String;
}
