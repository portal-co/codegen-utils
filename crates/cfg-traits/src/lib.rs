#![no_std]

use core::cmp::Ordering;
use core::hash::Hash;
#[doc(hidden)]
pub use core::ops::{Deref, DerefMut};
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
#[macro_export]
macro_rules! func_via_cfg {
    (<$($param:ident $([: $($path:path),*])?),*>$i:ident => $t:ty) => {
        pub struct $i<$($param : $($($path)+*)?),*>(pub $crate::FuncViaCfg<$t,Self>);
        const _: () = {
            impl<$($param : $($($path)+*)?),*> $crate::Deref for $i<$($param),*>{
                type Target = $crate::FuncViaCfg<$t,Self>;
                fn deref(&self) -> &$crate::FuncViaCfg<$t,Self>{
                    match self{
                        $i(a) => a,
                    }
                }
            }
        }
    };
}

pub struct FuncViaCfg<T, W: Deref<Target = Self> + Func + ?Sized> {
    pub cfg: T,
    pub entry_block: W::Block,
}
pub trait CfgOf: Func + Deref<Target = FuncViaCfg<Self::Cfg, Self>> {
    type Cfg;
}
impl<T, W: Deref<Target = FuncViaCfg<T, W>> + Func> CfgOf for W {
    type Cfg = T;
}
impl<T: Clone, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: Clone> + ?Sized> Clone
    for FuncViaCfg<T, W>
{
    fn clone(&self) -> Self {
        Self {
            cfg: self.cfg.clone(),
            entry_block: self.entry_block.clone(),
        }
    }
}
impl<T: PartialEq, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: PartialEq> + ?Sized> PartialEq
    for FuncViaCfg<T, W>
{
    fn eq(&self, other: &Self) -> bool {
        self.cfg == other.cfg && self.entry_block == other.entry_block
    }
}
impl<T: Eq, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: Eq> + ?Sized> Eq
    for FuncViaCfg<T, W>
{
}
impl<T: PartialOrd, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: PartialOrd> + ?Sized>
    PartialOrd for FuncViaCfg<T, W>
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        match self.cfg.partial_cmp(&other.cfg) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.entry_block.partial_cmp(&other.entry_block)
    }
}
impl<T: Ord, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: Ord> + ?Sized> Ord
    for FuncViaCfg<T, W>
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self.cfg.cmp(&other.cfg) {
            Ordering::Equal => return self.entry_block.cmp(&other.entry_block),
            a => return a,
        }
    }
}
impl<T: Hash, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: Hash> + ?Sized> Hash
    for FuncViaCfg<T, W>
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.cfg.hash(state);
        self.entry_block.hash(state);
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
