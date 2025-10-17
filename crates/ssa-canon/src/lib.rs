use std::iter::once;

// use anyhow::Ok;
use id_arena::{Arena, Id};
use sift_trait::Sift;
use ssa_traits::{op::OpValue, Term};
use cfg_traits::{Term as CFGTerm};

pub enum Value<O, T, Y> {
    Op(O, Vec<Id<Value<O, T, Y>>>, Vec<Id<Block<O, T, Y>>>, Y),
    Param(usize, Id<Block<O, T, Y>>, Y),
}
pub struct Block<O, T, Y> {
    pub term: T,
    pub insts: Vec<Id<Value<O, T, Y>>>,
    pub params: Vec<(Y, Id<Value<O, T, Y>>)>,
}
pub struct Target<O, T, Y> {
    pub args: Vec<Id<Value<O, T, Y>>>,
    pub block: Id<Block<O, T, Y>>,
}
pub struct Func<O, T, Y> {
    pub vals: Arena<Value<O, T, Y>>,
    pub blocks: Arena<Block<O, T, Y>>,
    pub entry: Id<Block<O, T, Y>>,
}
impl<O, T: Term<Func<O, T, Y>, Target = Target<O, T, Y>>, Y: Clone> cfg_traits::Func
    for Func<O, T, Y>
{


    type Block = Id<Block<O, T, Y>>;



    type Blocks = Arena<Block<O, T, Y>>;



    fn blocks(&self) -> &Self::Blocks {
        &self.blocks
    }



    fn blocks_mut(&mut self) -> &mut Self::Blocks {
        &mut self.blocks
    }

    fn entry(&self) -> Self::Block {
        self.entry
    }
}
impl<O, T: Term<Func<O, T, Y>, Target = Target<O, T, Y>>, Y: Clone> ssa_traits::Func
    for Func<O, T, Y>
{
    type Value = Id<Value<O, T, Y>>;



    type Values = Arena<Value<O, T, Y>>;



    fn values(&self) -> &Self::Values {
        &self.vals
    }



    fn values_mut(&mut self) -> &mut Self::Values {
        &mut self.vals
    }


}
impl<O, T: Term<Func<O, T, Y>, Target = Target<O, T, Y>>, Y: Clone> ssa_traits::TypedFunc
    for Func<O, T, Y>
{
    type Ty = Y;

    fn add_blockparam(&mut self, k: Self::Block, y: Self::Ty) -> Self::Value {
        let i = self.blocks[k].params.len();
        let v = self.vals.alloc(Value::Param(i, k, y.clone()));
        self.blocks[k].insts = vec![v]
            .into_iter()
            .chain(self.blocks[k].insts.iter().map(|a| *a))
            .collect();
        self.blocks[k].params.push((y.clone(), v));
        return v;
    }
}
impl<O, T: Term<Func<O, T, Y>, Target = Target<O, T, Y>>, Y: Clone>
    ssa_traits::HasValues<Func<O, T, Y>> for Value<O, T, Y>
{
    fn values<'a>(
        &'a self,
        f: &'a Func<O, T, Y>,
    ) -> Box<(dyn Iterator<Item = Id<Value<O, T, Y>>> + 'a)> {
        Box::new(match self {
            Value::Op(_, a, _, _) => Some(a.iter().cloned()),
            Value::Param(_, _, _) => None,
        }
        .into_iter()
        .flatten())
    }

    fn values_mut<'a>(
        &'a mut self,
        g: &'a mut Func<O, T, Y>,
    ) -> Box<(dyn Iterator<Item = &'a mut Id<Value<O, T, Y>>> + 'a)>
    where
        Func<O, T, Y>: 'a,
    {
        Box::new(match self {
            Value::Op(_, a, _, _) => Some(a.iter_mut()),
            Value::Param(_, _, _) => None,
        }
        .into_iter()
        .flatten())
    }
}

impl<O, T: Term<Func<O, T, Y>, Target = Target<O, T, Y>>, Y: Clone> ssa_traits::Value<Func<O, T, Y>>
    for Value<O, T, Y>
{
}

impl<O, T: Term<Func<O, T, Y>, Target = Target<O, T, Y>>, Y: Clone>
    ssa_traits::TypedValue<Func<O, T, Y>> for Value<O, T, Y>
{
    fn ty(&self, f: &Func<O, T, Y>) -> <Func<O, T, Y> as ssa_traits::TypedFunc>::Ty {
        match self {
            Value::Op(_, _, _, y) => y,
            Value::Param(_, _, y) => y,
        }
        .clone()
    }
}

impl<O, T: Term<Func<O, T, Y>, Target = Target<O, T, Y>>, Y: Clone> cfg_traits::Block<Func<O, T, Y>>
    for Block<O, T, Y>
{


    type Terminator = T;

    fn term(&self) -> &Self::Terminator {
        &self.term
    }

    fn term_mut(&mut self) -> &mut Self::Terminator {
        &mut self.term
    }
}

impl<O, T: Term<Func<O, T, Y>, Target = Target<O, T, Y>>, Y: Clone> ssa_traits::Block<Func<O, T, Y>>
    for Block<O, T, Y>
{
    fn insts(&self) -> impl Iterator<Item = <Func<O, T, Y> as ssa_traits::Func>::Value> {
        self.insts.iter().cloned()
    }

    fn add_inst(
        func: &mut Func<O, T, Y>,
        key: <Func<O, T, Y> as cfg_traits::Func>::Block,
        v: <Func<O, T, Y> as ssa_traits::Func>::Value,
    ) {
        func.blocks[key].insts.push(v);
    }

}


impl<O, T: Term<Func<O, T, Y>, Target = Target<O, T, Y>>, Y: Clone>
    ssa_traits::TypedBlock<Func<O, T, Y>> for Block<O, T, Y>
{
    fn params(
        &self,
    ) -> impl Iterator<
        Item = (
            <Func<O, T, Y> as ssa_traits::TypedFunc>::Ty,
            Id<Value<O, T, Y>>,
        ),
    > {
        self.params.iter().cloned()
    }
}
impl<O, T: Term<Func<O, T, Y>, Target = Target<O, T, Y>>, Y: Clone>
    ssa_traits::HasValues<Func<O, T, Y>> for Target<O, T, Y>
{
    fn values<'a>(
        &'a self,
        f: &'a Func<O, T, Y>,
    ) -> Box<(dyn Iterator<Item = Id<Value<O, T, Y>>> + 'a)> {
        Box::new(self.args.iter().cloned())
    }

    fn values_mut<'a>(
        &'a mut self,
        g: &'a mut Func<O, T, Y>,
    ) -> Box<(dyn Iterator<Item = &'a mut Id<Value<O, T, Y>>> + 'a)>
    where
        Func<O, T, Y>: 'a,
    {
        Box::new(self.args.iter_mut())
    }
}
impl<O, T: Term<Func<O, T, Y>, Target = Target<O, T, Y>>, Y: Clone> cfg_traits::Term<Func<O, T, Y>>
    for Target<O, T, Y>
{
    type Target = Self;

    fn targets<'a>(&'a self) -> Box<(dyn Iterator<Item = &'a Target<O, T, Y>> + 'a)>
    where
        Func<O, T, Y>: 'a,
    {
        Box::new(once(self))
    }

    fn targets_mut<'a>(&'a mut self) -> Box<(dyn Iterator<Item = &'a mut Target<O, T, Y>> + 'a)>
    where
        Func<O, T, Y>: 'a,
    {
        Box::new(once(self))
    }
}
impl<O, T: Term<Func<O, T, Y>, Target = Target<O, T, Y>>, Y: Clone>
    cfg_traits::Target<Func<O, T, Y>> for Target<O, T, Y>
{
    fn block(&self) -> <Func<O, T, Y> as cfg_traits::Func>::Block {
        self.block
    }

    fn block_mut(&mut self) -> &mut <Func<O, T, Y> as cfg_traits::Func>::Block {
        &mut self.block
    }

}

impl<O, T: Term<Func<O, T, Y>, Target = Target<O, T, Y>>, Y: Clone>
    ssa_traits::Target<Func<O, T, Y>> for Target<O, T, Y>
{

    fn push_value(&mut self, v: <Func<O, T, Y> as ssa_traits::Func>::Value) {
        self.args.push(v);
    }

    fn from_values_and_block(
        a: impl Iterator<Item = <Func<O, T, Y> as ssa_traits::Func>::Value>,
        k: <Func<O, T, Y> as cfg_traits::Func>::Block,
    ) -> Self {
        Target {
            args: a.collect(),
            block: k,
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonOp<T> {
    pub op: T,
}
impl<O: Sift<X>, T: Term<Func<O, T, Y>, Target = Target<O, T, Y>>, Y: Clone, X>
    OpValue<Func<O, T, Y>, CanonOp<X>> for Value<O, T, Y>
{
    type Residue = Value<<O as Sift<X>>::Residue, T, Y>;

    type Capture = Vec<Id<Value<O, T, Y>>>;

    type Spit = (Y, Vec<Id<Block<O, T, Y>>>);

    fn disasm(
        self,
        f: &mut Func<O, T, Y>,
    ) -> Result<
        (
            CanonOp<X>,
            Vec<Id<Value<O, T, Y>>>,
            (Y, Vec<Id<Block<O, T, Y>>>),
        ),
        Value<<O as Sift<X>>::Residue, T, Y>,
    > {
        match self {
            Value::Op(o, p, q, y) => match o.sift() {
                Ok(px) => Ok((CanonOp { op: px }, p, (y, q))),
                Err(r) => Err(Value::Op(
                    r,
                    p.into_iter()
                        .map(|a| unsafe { std::mem::transmute(a) })
                        .collect(),
                    q.into_iter()
                        .map(|a| unsafe { std::mem::transmute(a) })
                        .collect(),
                    y,
                )),
            },
            Value::Param(a, b, c) => Err(Value::Param(a, unsafe { std::mem::transmute(b) }, c)),
        }
    }

    fn of(f: &mut Func<O, T, Y>, o: CanonOp<X>, c: Self::Capture, s: Self::Spit) -> Option<Self> {
        Some(Value::Op(<O as Sift<X>>::of(o.op), c, s.1, s.0))
    }

    fn lift(f: &mut Func<O, T, Y>, r: Self::Residue) -> Option<Self> {
        Some(match r {
            Value::Op(o, p, q, y) => {
                let r = <O as Sift<X>>::lift(o);
                Value::Op(
                    r,
                    p.into_iter()
                        .map(|a| unsafe { std::mem::transmute(a) })
                        .collect(),
                    q.into_iter()
                        .map(|a| unsafe { std::mem::transmute(a) })
                        .collect(),
                    y,
                )
            }
            Value::Param(a, b, c) => Value::Param(a, unsafe { std::mem::transmute(b) }, c),
        })
    }
}
