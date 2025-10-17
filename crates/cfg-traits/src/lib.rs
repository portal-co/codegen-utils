#![no_std]

use core::cmp::Ordering;
use core::hash::Hash;
use core::ops::{Deref, DerefMut};
use core::{iter::once, ops::Index};

extern crate alloc;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use arena_traits::Arena;
use either::Either;
use lending_iterator::lending_iterator::constructors::into_lending_iter;
use lending_iterator::prelude::{LendingIteratorDyn, HKT};
use lending_iterator::LendingIterator;

pub struct FuncViaCfg<T, W: Deref<Target = Self> + Func> {
    pub cfg: T,
    pub block: W::Block,
}
impl<T: Clone, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: Clone>> Clone
    for FuncViaCfg<T, W>
{
    fn clone(&self) -> Self {
        Self {
            cfg: self.cfg.clone(),
            block: self.block.clone(),
        }
    }
}
impl<T: PartialEq, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: PartialEq>> PartialEq
    for FuncViaCfg<T, W>
{
    fn eq(&self, other: &Self) -> bool {
        self.cfg == other.cfg && self.block == other.block
    }
}
impl<T: Eq, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: Eq>> Eq for FuncViaCfg<T, W> {}
impl<T: PartialOrd, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: PartialOrd>> PartialOrd
    for FuncViaCfg<T, W>
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        match self.cfg.partial_cmp(&other.cfg) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.block.partial_cmp(&other.block)
    }
}
impl<T: Ord, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: Ord>> Ord for FuncViaCfg<T, W> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self.cfg.cmp(&other.cfg) {
            Ordering::Equal => return self.block.cmp(&other.block),
            a => return a,
        }
    }
}
impl<T: Hash, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: Hash>> Hash for FuncViaCfg<T, W> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.cfg.hash(state);
        self.block.hash(state);
    }
}

// pub mod op;
pub trait Func {
    // type Value;
    type Block;
    // type Values: Arena<Self::Value, Output: Value<Self>>;
    type Blocks: Arena<Self::Block, Output: Block<Self>>;
    type BRef<'a>: Deref<Target = Self::Blocks>
    where
        Self: 'a;
    type BMut<'a>: DerefMut<Target = Self::Blocks>
    where
        Self: 'a;
    // fn values(&self) -> &Self::Values;
    fn blocks<'a>(&'a self) -> Self::BRef<'a>;
    // fn values_mut(&mut self) -> &mut Self::Values;
    fn blocks_mut<'a>(&'a mut self) -> Self::BMut<'a>;
    fn entry(&self) -> Self::Block;
}
// pub type ValueI<F> = <<F as Func>::Values as Index<<F as Func>::Value>>::Output;
pub type BlockI<F> = <<F as Func>::Blocks as Index<<F as Func>::Block>>::Output;
pub type TermI<F> = <BlockI<F> as Block<F>>::Terminator;
pub type TargetI<F> = <TermI<F> as Term<F>>::Target;

pub trait Block<F: Func<Blocks: Arena<F::Block, Output = Self>> + ?Sized> {
    type Terminator: Term<F>;
    fn term(&self) -> &Self::Terminator;
    fn term_mut(&mut self) -> &mut Self::Terminator;
}

pub trait Target<F: Func + ?Sized>: Term<F, Target = Self> {
    fn block(&self) -> F::Block;
    type BMut<'a>: DerefMut<Target = F::Block>
    where
        Self: 'a;
    fn block_mut<'a>(&'a mut self) -> Self::BMut<'a>;
}
pub trait Term<F: Func + ?Sized> {
    type Target: Target<F>;
    fn targets<'a>(
        &'a self,
    ) -> Box<
        dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn Deref<Target = Self::Target> + 'b>)>
            + 'a,
    >
    where
        F: 'a;
    fn targets_mut<'a>(
        &'a mut self,
    ) -> Box<
        dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn DerefMut<Target = Self::Target> + 'b>)>
            + 'a,
    >
    where
        F: 'a;
}

impl<F: Func + ?Sized, T: Target<F>, A: Term<F, Target = T>, B: Term<F, Target = T>> Term<F>
    for Either<A, B>
{
    type Target = T;

    fn targets<'a>(
        &'a self,
    ) -> Box<
        dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn Deref<Target = Self::Target> + 'b>)>
            + 'a,
    >
    where
        F: 'a,
    {
        match self {
            Either::Left(a) => a.targets(),
            Either::Right(b) => b.targets(),
        }
    }

    fn targets_mut<'a>(
        &'a mut self,
    ) -> Box<
        dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn DerefMut<Target = Self::Target> + 'b>)>
            + 'a,
    >
    where
        F: 'a,
    {
        match self {
            Either::Left(a) => a.targets_mut(),
            Either::Right(b) => b.targets_mut(),
        }
    }
}
pub fn val_iter<'a, V: 'a, I: Iterator<Item: Deref<Target = V> + 'a> + 'a>(
    i: I,
) -> Box<dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn Deref<Target = V> + 'b>)> + 'a> {
    Box::new(
        i.into_lending_iter()
            .map::<HKT!(<'b> => Box<dyn Deref<Target = V> + 'b>), _>(|[], x| Box::new(x)),
    )
}
pub fn val_mut_iter<'a, V: 'a, I: Iterator<Item: DerefMut<Target = V> + 'a> + 'a>(
    i: I,
) -> Box<dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn DerefMut<Target = V> + 'b>)> + 'a> {
    Box::new(
        i.into_lending_iter()
            .map::<HKT!(<'b> => Box<dyn DerefMut<Target = V> + 'b>), _>(|[], x| Box::new(x)),
    )
}
