use std::iter::once;

use id_arena::{Arena, Id};
use ssa_traits::{Term};

pub enum Value<O, T, Y> {
    Op(O, Vec<Id<Value<O, T, Y>>>, Y),
    Param(usize, Id<Block<O, T, Y>>, Y),
}
pub struct Block<O, T, Y> {
    pub term: T,
    pub insts: Vec<Id<Value<O, T, Y>>>,
    pub params: Vec<Y>,
}
pub struct Target<O,T,Y>{
    pub args: Vec<Id<Value<O,T,Y>>>,
    pub block: Id<Block<O,T,Y>>
}
pub struct Func<O, T, Y> {
    pub vals: Arena<Value<O, T, Y>>,
    pub blocks: Arena<Block<O, T, Y>>,
    pub entry: Id<Block<O, T, Y>>,
}
impl<O,T: Term<Func<O,T,Y>,Target = Target<O,T,Y>>,Y: Clone> ssa_traits::Func for Func<O,T,Y>{
    type Value = Id<Value<O,T,Y>>;

    type Block = Id<Block<O,T,Y>>;

    type Values = Arena<Value<O, T, Y>>;

    type Blocks = Arena<Block<O, T, Y>>;

    fn values(&self) -> &Self::Values {
        &self.vals
    }

    fn blocks(&self) -> &Self::Blocks {
      &self.blocks
    }

    fn values_mut(&mut self) -> &mut Self::Values {
        &mut self.vals
    }

    fn blocks_mut(&mut self) -> &mut Self::Blocks {
        &mut self.blocks
    }

    fn entry(&self) -> Self::Block {
        self.entry
    }
}
impl<O,T: Term<Func<O,T,Y>,Target = Target<O,T,Y>>,Y: Clone> ssa_traits::TypedFunc for Func<O,T,Y>{
    type Ty = Y;

    fn add_blockparam(&mut self, k: Self::Block, y: Self::Ty) -> Self::Value {
        let i = self.blocks[k].params.len();
        self.blocks[k].params.push(y.clone());
        let v = self.vals.alloc(Value::Param(i, k, y));
        self.blocks[k].insts = vec![v]
            .into_iter()
            .chain(self.blocks[k].insts.iter().map(|a| *a))
            .collect();
        return v;
    }
}
impl<O,T: Term<Func<O,T,Y>,Target = Target<O,T,Y>>,Y: Clone> ssa_traits::HasValues<Func<O,T,Y>> for Value<O,T,Y>{
    fn values(&self, f: &Func<O,T,Y>) -> impl Iterator<Item = <Func<O,T,Y> as ssa_traits::Func>::Value> {
        match self{
            Value::Op(_, a, _) => Some(a.iter().cloned()),
            Value::Param(_, _, _) => None,
        }.into_iter().flatten()
    }

    fn values_mut<'a>(&'a mut self, g: &'a mut Func<O,T,Y>) -> impl Iterator<Item = &'a mut <Func<O,T,Y> as ssa_traits::Func>::Value>
    where
        Func<O,T,Y>: 'a {
            match self{
                Value::Op(_, a, _) => Some(a.iter_mut()),
                Value::Param(_, _, _) => None,
            }.into_iter().flatten()
    }
}

impl<O,T: Term<Func<O,T,Y>,Target = Target<O,T,Y>>,Y: Clone> ssa_traits::Value<Func<O,T,Y>> for Value<O,T,Y>{

}

impl<O,T: Term<Func<O,T,Y>,Target = Target<O,T,Y>>,Y: Clone> ssa_traits::TypedValue<Func<O,T,Y>> for Value<O,T,Y>{
    fn ty(&self, f: &Func<O,T,Y>) -> <Func<O,T,Y> as ssa_traits::TypedFunc>::Ty {
        match self{
            Value::Op(_, _, y) => y,
            Value::Param(_, _, y) =>y,
        }.clone()
    }
}

impl<O,T: Term<Func<O,T,Y>,Target = Target<O,T,Y>>,Y: Clone> ssa_traits::Block<Func<O,T,Y>> for Block<O,T,Y>{
    fn insts(&self) -> impl Iterator<Item = <Func<O,T,Y> as ssa_traits::Func>::Value> {
        self.insts.iter().cloned()
    }

    fn add_inst(func: &mut Func<O,T,Y>,key:<Func<O,T,Y> as ssa_traits::Func>::Block, v: <Func<O,T,Y> as ssa_traits::Func>::Value) {
        func.blocks[key].insts.push(v)
        ;
    }

    type Terminator = T;

    fn term(&self) -> &Self::Terminator {
        &self.term
    }

    fn term_mut(&mut self) -> &mut Self::Terminator {
        &mut self.term
    }
}

impl<O,T: Term<Func<O,T,Y>,Target = Target<O,T,Y>>,Y: Clone> ssa_traits::TypedBlock<Func<O,T,Y>> for Block<O,T,Y>{
    fn params(&self) -> impl Iterator<Item = <Func<O,T,Y> as ssa_traits::TypedFunc>::Ty> {
        self.params.iter().cloned()
    }
}
impl<O,T: Term<Func<O,T,Y>,Target = Target<O,T,Y>>,Y: Clone> ssa_traits::HasValues<Func<O,T,Y>> for Target<O,T,Y>{
    fn values(&self, f: &Func<O,T,Y>) -> impl Iterator<Item = <Func<O,T,Y> as ssa_traits::Func>::Value> {
        self.args.iter().cloned()
    }

    fn values_mut<'a>(&'a mut self, g: &'a mut Func<O,T,Y>) -> impl Iterator<Item = &'a mut <Func<O,T,Y> as ssa_traits::Func>::Value>
    where
        Func<O,T,Y>: 'a {
        self.args.iter_mut()
    }
}
impl<O,T: Term<Func<O,T,Y>,Target = Target<O,T,Y>>,Y: Clone> ssa_traits::Term<Func<O,T,Y>> for Target<O,T,Y>{
    type Target = Self;

    fn targets<'a>(&'a self) -> impl Iterator<Item = &'a Self::Target>
    where
        Func<O,T,Y>: 'a {
        once(self)
    }

    fn targets_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Self::Target>
    where
        Func<O,T,Y>: 'a {
        once(self)
    }
}
impl<O,T: Term<Func<O,T,Y>,Target = Target<O,T,Y>>,Y: Clone> ssa_traits::Target<Func<O,T,Y>> for Target<O,T,Y>{
    fn block(&self) -> <Func<O,T,Y> as ssa_traits::Func>::Block {
        self.block
    }

    fn block_mut(&mut self) -> &mut <Func<O,T,Y> as ssa_traits::Func>::Block {
        &mut self.block
    }

    fn push_value(&mut self, v: <Func<O,T,Y> as ssa_traits::Func>::Value) {
        self.args.push(v);
    }

    fn from_values_and_block(a: impl Iterator<Item = <Func<O,T,Y> as ssa_traits::Func>::Value>, k: <Func<O,T,Y> as ssa_traits::Func>::Block) -> Self {
        Target { args: a.collect(), block: k }
    }
}