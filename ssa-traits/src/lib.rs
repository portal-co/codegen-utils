use std::collections::BTreeMap;

use arena_traits::Arena;
use either::Either;

pub trait Func {
    type Value;
    type Block;
    type Values: Arena<Self::Value, Output: Value<Self>>;
    type Blocks: Arena<Self::Block, Output: Block<Self>>;
    fn values(&self) -> &Self::Values;
    fn blocks(&self) -> &Self::Blocks;
    fn values_mut(&mut self) -> &mut Self::Values;
    fn blocks_mut(&mut self) -> &mut Self::Blocks;
}
pub trait Builder<F: Func> {
    type Result;
    fn build(self, f: &mut F, k: F::Block) -> anyhow::Result<(Self::Result, F::Block)>;
}
pub trait Block<F: Func<Blocks: Arena<F::Block, Output = Self>> + ?Sized> {
    fn insts(&self) -> impl Iterator<Item = F::Value>;
    fn add_inst(&mut self, v: F::Value);
    type Terminator: Term<F>;
    fn term(&self) -> &Self::Terminator;
    fn term_mut(&mut self) -> &mut Self::Terminator;
}
pub trait Value<F: Func<Values: Arena<F::Value, Output = Self>> + ?Sized>: HasValues<F> {}
pub trait TypedValue<F: TypedFunc<Values: Arena<F::Value, Output = Self>> + ?Sized>:
    Value<F>
{
    fn ty(&self) -> F::Ty;
}
pub trait TypedFunc:
    Func<
    Values: Arena<Self::Value, Output: TypedValue<Self>>,
    Blocks: Arena<Self::Block, Output: TypedBlock<Self>>,
>
{
    type Ty;
    fn add_blockparam(&mut self, k: Self::Block, y: Self::Ty) -> Self::Value;
}
pub trait TypedBlock<F: TypedFunc<Blocks: Arena<F::Block, Output = Self>> + ?Sized>:
    Block<F>
{
    fn params(&self) -> impl Iterator<Item = F::Ty>;
}

pub trait HasValues<F: Func + ?Sized> {
    fn values(&self) -> impl Iterator<Item = F::Value>;
    fn values_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut F::Value>
    where
        F: 'a;
}
impl<F: Func + ?Sized, A: HasValues<F>, B: HasValues<F>> HasValues<F> for Either<A, B> {
    fn values(&self) -> impl Iterator<Item = <F as Func>::Value> {
        match self {
            Either::Left(a) => Either::Left(a.values()),
            Either::Right(b) => Either::Right(b.values()),
        }
    }

    fn values_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut <F as Func>::Value>
    where
        F: 'a,
    {
        match self {
            Either::Left(a) => Either::Left(a.values_mut()),
            Either::Right(b) => Either::Right(b.values_mut()),
        }
    }
}
pub trait Target<F: Func + ?Sized>: Term<F, Target = Self> {
    fn block(&self) -> F::Block;
    fn block_mut(&mut self) -> &mut F::Block;
    fn from_values_and_block(a: impl Iterator<Item = F::Value>, k: F::Block) -> Self;
}
pub trait Term<F: Func + ?Sized>: HasValues<F> {
    type Target: Target<F>;
    fn targets<'a>(&'a self) -> impl Iterator<Item = &'a Self::Target>
    where
        F: 'a;
    fn targets_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Self::Target>
    where
        F: 'a;
}
impl<F: Func + ?Sized, T: Target<F>, A: Term<F, Target = T>, B: Term<F, Target = T>> Term<F>
    for Either<A, B>
{
    type Target = T;

    fn targets<'a>(&'a self) -> impl Iterator<Item = &'a Self::Target>
    where
        F: 'a,
    {
        match self {
            Either::Left(a) => Either::Left(a.targets()),
            Either::Right(b) => Either::Right(b.targets()),
        }
    }

    fn targets_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Self::Target>
    where
        F: 'a,
    {
        match self {
            Either::Left(a) => Either::Left(a.targets_mut()),
            Either::Right(b) => Either::Right(b.targets_mut()),
        }
    }
}
