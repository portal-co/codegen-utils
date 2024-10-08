#![no_std]

use core::{iter::once, ops::Index};

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use arena_traits::Arena;
use either::Either;
pub mod op;
pub trait Func {
    type Value;
    type Block;
    type Values: Arena<Self::Value, Output: Value<Self>>;
    type Blocks: Arena<Self::Block, Output: Block<Self>>;
    fn values(&self) -> &Self::Values;
    fn blocks(&self) -> &Self::Blocks;
    fn values_mut(&mut self) -> &mut Self::Values;
    fn blocks_mut(&mut self) -> &mut Self::Blocks;
    fn entry(&self) -> Self::Block;
}
pub type ValueI<F> = <<F as Func>::Values as Index<<F as Func>::Value>>::Output;
pub type BlockI<F> = <<F as Func>::Blocks as Index<<F as Func>::Block>>::Output;
pub type TermI<F> = <BlockI<F> as Block<F>>::Terminator;
pub type TargetI<F> = <TermI<F> as Term<F>>::Target;

#[repr(transparent)]
pub struct Val<F: Func + ?Sized>(pub F::Value);
impl<F: Func<Value: Clone> + ?Sized> Clone for Val<F> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<F: Func<Value: Clone> + ?Sized> HasValues<F> for Val<F> {
    fn values(&self, f: &F) -> impl Iterator<Item = <F as Func>::Value> {
        once(self.0.clone())
    }

    fn values_mut<'a>(
        &'a mut self,
        g: &'a mut F,
    ) -> impl Iterator<Item = &'a mut <F as Func>::Value>
    where
        F: 'a,
    {
        once(&mut self.0)
    }
}
impl<F: Func<Value: Clone> + ?Sized> HasChainableValues<F> for Val<F> {
    fn values_chain(&self) -> impl Iterator<Item = <F as Func>::Value> {
        once(self.0.clone())
    }

    fn values_chain_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut <F as Func>::Value>
    where
        F: 'a,
    {
        once(&mut self.0)
    }
}
impl<F: Func<Value: Clone> + ?Sized> HasValues<F> for Vec<F::Value> {
    fn values(&self, f: &F) -> impl Iterator<Item = <F as Func>::Value> {
        self.iter().cloned()
    }

    fn values_mut<'a>(
        &'a mut self,
        g: &'a mut F,
    ) -> impl Iterator<Item = &'a mut <F as Func>::Value>
    where
        F: 'a,
    {
        self.iter_mut()
    }
}
impl<F: Func<Value: Clone> + ?Sized> HasChainableValues<F> for Vec<F::Value> {
    fn values_chain(&self) -> impl Iterator<Item = <F as Func>::Value> {
        self.iter().cloned()
    }

    fn values_chain_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut <F as Func>::Value>
    where
        F: 'a,
    {
        self.iter_mut()
    }
}
pub struct BuildFn<F> {
    pub func: F,
}
pub fn build_fn<F: FnOnce(&mut G, G::Block) -> anyhow::Result<(R, G::Block)>, G: Func, R>(
    f: F,
) -> BuildFn<F> {
    BuildFn { func: f }
}
pub trait Builder<F: Func> {
    type Result;
    fn build(self, f: &mut F, k: F::Block) -> anyhow::Result<(Self::Result, F::Block)>;
}
impl<F: Func, B: Builder<F>> Builder<F> for anyhow::Result<B> {
    type Result = B::Result;

    fn build(
        self,
        f: &mut F,
        k: <F as Func>::Block,
    ) -> anyhow::Result<(Self::Result, <F as Func>::Block)> {
        self?.build(f, k)
    }
}
impl<F: FnOnce(&mut G, G::Block) -> anyhow::Result<(R, G::Block)>, G: Func, R> Builder<G> for F {
    type Result = R;

    fn build(
        self,
        f: &mut G,
        k: <G as Func>::Block,
    ) -> anyhow::Result<(Self::Result, <G as Func>::Block)> {
        self(f, k)
    }
}
pub trait Block<F: Func<Blocks: Arena<F::Block, Output = Self>> + ?Sized> {
    fn insts(&self) -> impl Iterator<Item = F::Value>;
    fn add_inst(func: &mut F, key: F::Block, v: F::Value);
    type Terminator: Term<F>;
    fn term(&self) -> &Self::Terminator;
    fn term_mut(&mut self) -> &mut Self::Terminator;
}
pub trait Value<F: Func<Values: Arena<F::Value, Output = Self>> + ?Sized>: HasValues<F> {}

pub trait TypedValue<F: TypedFunc<Values: Arena<F::Value, Output = Self>> + ?Sized>:
    Value<F>
{
    fn ty(&self, f: &F) -> F::Ty;
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
    fn params(&self) -> impl Iterator<Item = (F::Ty, F::Value)>;
}

pub trait HasValues<F: Func + ?Sized> {
    fn values(&self, f: &F) -> impl Iterator<Item = F::Value>;
    fn values_mut<'a>(&'a mut self, g: &'a mut F) -> impl Iterator<Item = &'a mut F::Value>
    where
        F: 'a;
}
pub trait FromValues<F: Func + ?Sized>: HasValues<F> {
    fn from_values(f: &mut F, i: impl Iterator<Item = F::Value>) -> Self;
}
pub trait HasChainableValues<F: Func + ?Sized>: HasValues<F> {
    fn values_chain(&self) -> impl Iterator<Item = F::Value>;
    fn values_chain_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut F::Value>
    where
        F: 'a;
}
impl<F: Func + ?Sized, A: HasValues<F>, B: HasValues<F>> HasValues<F> for Either<A, B> {
    fn values(&self, f: &F) -> impl Iterator<Item = <F as Func>::Value> {
        match self {
            Either::Left(a) => Either::Left(a.values(f)),
            Either::Right(b) => Either::Right(b.values(f)),
        }
    }

    fn values_mut<'a>(
        &'a mut self,
        f: &'a mut F,
    ) -> impl Iterator<Item = &'a mut <F as Func>::Value>
    where
        F: 'a,
    {
        match self {
            Either::Left(a) => Either::Left(a.values_mut(f)),
            Either::Right(b) => Either::Right(b.values_mut(f)),
        }
    }
}
impl<F: Func + ?Sized, A: HasChainableValues<F>, B: HasChainableValues<F>> HasChainableValues<F>
    for Either<A, B>
{
    fn values_chain(&self) -> impl Iterator<Item = <F as Func>::Value> {
        match self {
            Either::Left(a) => Either::Left(a.values_chain()),
            Either::Right(b) => Either::Right(b.values_chain()),
        }
    }

    fn values_chain_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut <F as Func>::Value>
    where
        F: 'a,
    {
        match self {
            Either::Left(a) => Either::Left(a.values_chain_mut()),
            Either::Right(b) => Either::Right(b.values_chain_mut()),
        }
    }
}
pub trait Target<F: Func + ?Sized>: Term<F, Target = Self> {
    fn block(&self) -> F::Block;
    fn block_mut(&mut self) -> &mut F::Block;
    fn push_value(&mut self, v: F::Value);
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
