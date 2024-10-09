#![no_std]

use core::{iter::once, ops::Index};

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use arena_traits::Arena;
use either::Either;
// pub mod op;
pub trait Func {
    // type Value;
    type Block;
    // type Values: Arena<Self::Value, Output: Value<Self>>;
    type Blocks: Arena<Self::Block, Output: Block<Self>>;
    // fn values(&self) -> &Self::Values;
    fn blocks(&self) -> &Self::Blocks;
    // fn values_mut(&mut self) -> &mut Self::Values;
    fn blocks_mut(&mut self) -> &mut Self::Blocks;
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
    fn block_mut(&mut self) -> &mut F::Block;
}
pub trait Term<F: Func + ?Sized> {
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