#![no_std]

use core::ops::{Deref, DerefMut};
use core::slice::{Iter, IterMut};
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
pub mod op;
pub use cfg_traits::{val_iter,val_mut_iter};
pub trait Func: cfg_traits::Func<Blocks: Arena<Self::Block, Output: Block<Self>>> {
    type Value;
    type Values: Arena<Self::Value, Output: Value<Self>>;
    type VRef<'a>: Deref<Target = Self::Values>
    where
        Self: 'a;
    type VMut<'a>: DerefMut<Target = Self::Values>
    where
        Self: 'a;
    fn values<'a>(&'a self) -> Self::VRef<'a>;
    fn values_mut<'a>(&'a mut self) -> Self::VMut<'a>;
}
pub type ValueI<F> = <<F as Func>::Values as Index<<F as Func>::Value>>::Output;
pub type BlockI<F> =
    <<F as cfg_traits::Func>::Blocks as Index<<F as cfg_traits::Func>::Block>>::Output;
pub type TermI<F> = <BlockI<F> as cfg_traits::Block<F>>::Terminator;
pub type TargetI<F> = <TermI<F> as cfg_traits::Term<F>>::Target;

#[repr(transparent)]
pub struct Val<F: Func + ?Sized>(pub F::Value);
impl<F: Func<Value: Clone> + ?Sized> Clone for Val<F> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<F: Func<Value: Clone> + ?Sized> HasValues<F> for Val<F> {
    fn values<'a>(
        &'a self,
        f: &'a F,
    ) -> Box<dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn Deref<Target = F::Value> + 'b>)> + 'a>
    {
        Box::new(lending_iterator::lending_iterator::constructors::from_fn::<
            HKT!(<'b> => Box<dyn Deref<Target = F::Value> + 'b>),
            Option<&'a Self>,
            _,
        >(Some(self), |mut x| {
            let x = x.take()?;
            return Some(Box::new(&x.0));
        }))
    }

    fn values_mut<'a>(
        &'a mut self,
        g: &'a mut F,
    ) -> Box<
        (dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn DerefMut<Target = F::Value> + 'b>)>
             + 'a),
    >
    where
        F: 'a,
    {
        Box::new(lending_iterator::lending_iterator::constructors::from_fn::<
            HKT!(<'b> => Box<dyn DerefMut<Target = F::Value> + 'b>),
            Option<&'a mut Self>,
            _,
        >(Some(self), |mut x| {
            let x = x.take()?;
            return Some(Box::new(&mut x.0));
        }))
    }
}
impl<F: Func<Value: Clone> + ?Sized> HasChainableValues<F> for Val<F> {
    fn values_chain<'a>(
        &'a self,
    ) -> Box<dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn Deref<Target = F::Value> + 'b>)> + 'a>
    {
        Box::new(lending_iterator::lending_iterator::constructors::from_fn::<
            HKT!(<'b> => Box<dyn Deref<Target = F::Value> + 'b>),
            Option<&'a Self>,
            _,
        >(Some(self), |mut x| {
            let x = x.take()?;
            return Some(Box::new(&x.0));
        }))
    }

    fn values_chain_mut<'a>(
        &'a mut self,
    ) -> Box<
        dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn DerefMut<Target = F::Value> + 'b>)> + 'a,
    >
    where
        F: 'a,
    {
        Box::new(lending_iterator::lending_iterator::constructors::from_fn::<
            HKT!(<'b> => Box<dyn DerefMut<Target = F::Value> + 'b>),
            Option<&'a mut Self>,
            _,
        >(Some(self), |mut x| {
            let x = x.take()?;
            return Some(Box::new(&mut x.0));
        }))
    }
}
impl<F: Func<Value: Clone> + ?Sized> HasValues<F> for Vec<F::Value> {
    fn values<'a>(
        &'a self,
        f: &'a F,
    ) -> Box<dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn Deref<Target = F::Value> + 'b>)> + 'a>
    {
        Box::new(lending_iterator::lending_iterator::constructors::from_fn::<
            HKT!(<'b> => Box<dyn Deref<Target = F::Value> + 'b>),
            Option<Iter<'a, F::Value>>,
            _,
        >(Some(self.iter()), |mut x2| {
            let mut x = x2.take()?;
            let n = x.next()?;
            *x2 = Some(x);
            return Some(Box::new(n));
        }))
    }

    fn values_mut<'a>(
        &'a mut self,
        g: &'a mut F,
    ) -> Box<
        dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn DerefMut<Target = F::Value> + 'b>)> + 'a,
    >
    where
        F: 'a,
    {
        Box::new(lending_iterator::lending_iterator::constructors::from_fn::<
            HKT!(<'b> =>  Box<dyn DerefMut<Target = F::Value> + 'b>),
            Option<IterMut<'a, F::Value>>,
            _,
        >(Some(self.iter_mut()), |mut x2| {
            let mut x = x2.take()?;
            let n = x.next()?;
            *x2 = Some(x);
            return Some(Box::new(n));
        }))
    }
}
impl<F: Func<Value: Clone> + ?Sized> HasChainableValues<F> for Vec<F::Value> {
    fn values_chain<'a>(
        &'a self,
    ) -> Box<dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn Deref<Target = F::Value> + 'b>)> + 'a>
    {
        Box::new(lending_iterator::lending_iterator::constructors::from_fn::<
            HKT!(<'b> => Box<dyn Deref<Target = F::Value> + 'b>),
            Option<Iter<'a, F::Value>>,
            _,
        >(Some(self.iter()), |mut x2| {
            let mut x = x2.take()?;
            let n = x.next()?;
            *x2 = Some(x);
            return Some(Box::new(n));
        }))
    }
    fn values_chain_mut<'a>(
        &'a mut self,
    ) -> Box<
        dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn DerefMut<Target = F::Value> + 'b>)> + 'a,
    >
    where
        F: 'a,
    {
        Box::new(lending_iterator::lending_iterator::constructors::from_fn::<
            HKT!(<'b> =>  Box<dyn DerefMut<Target = F::Value> + 'b>),
            Option<IterMut<'a, F::Value>>,
            _,
        >(Some(self.iter_mut()), |mut x2| {
            let mut x = x2.take()?;
            let n = x.next()?;
            *x2 = Some(x);
            return Some(Box::new(n));
        }))
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
pub trait CpsBuilder<F: Func> {
    type Result;
    fn go<'a: 'b + 'c, 'b, 'c, R>(
        self,
        f: &'b mut F,
        k: F::Block,
        next: Box<dyn FnMut(&mut F, Self::Result, F::Block) -> anyhow::Result<R> + 'c>,
    ) -> Box<dyn Iterator<Item = anyhow::Result<R>> + 'a>;
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
        k: <F as cfg_traits::Func>::Block,
    ) -> anyhow::Result<(Self::Result, <F as cfg_traits::Func>::Block)> {
        self?.build(f, k)
    }
}
impl<F: FnOnce(&mut G, G::Block) -> anyhow::Result<(R, G::Block)>, G: Func, R> Builder<G> for F {
    type Result = R;

    fn build(
        self,
        f: &mut G,
        k: <G as cfg_traits::Func>::Block,
    ) -> anyhow::Result<(Self::Result, <G as cfg_traits::Func>::Block)> {
        self(f, k)
    }
}
pub trait Block<F: Func<Blocks: Arena<F::Block, Output = Self>> + ?Sized>:
    cfg_traits::Block<F, Terminator: Term<F>>
{
    fn insts(&self) -> impl Iterator<Item = F::Value>;
    fn add_inst(func: &mut F, key: F::Block, v: F::Value);
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
    fn values<'a>(
        &'a self,
        f: &'a F,
    ) -> Box<dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn Deref<Target = F::Value> + 'b>)> + 'a>;
    fn values_mut<'a>(
        &'a mut self,
        g: &'a mut F,
    ) -> Box<
        dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn DerefMut<Target = F::Value> + 'b>)> + 'a,
    >
    where
        F: 'a;
}
pub trait FromValues<F: Func + ?Sized>: HasValues<F> {
    fn from_values(f: &mut F, i: impl Iterator<Item = F::Value>) -> Self;
}
pub trait HasChainableValues<F: Func + ?Sized>: HasValues<F> {
    fn values_chain<'a>(
        &'a self,
    ) -> Box<dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn Deref<Target = F::Value> + 'b>)> + 'a>;
    fn values_chain_mut<'a>(
        &'a mut self,
    ) -> Box<
        dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn DerefMut<Target = F::Value> + 'b>)> + 'a,
    >
    where
        F: 'a;
}
impl<F: Func + ?Sized, A: HasValues<F>, B: HasValues<F>> HasValues<F> for Either<A, B> {
    fn values<'a>(
        &'a self,
        f: &'a F,
    ) -> Box<dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn Deref<Target = F::Value> + 'b>)> + 'a>
    {
        match self {
            Either::Left(a) => a.values(f),
            Either::Right(b) => b.values(f),
        }
    }

    fn values_mut<'a>(
        &'a mut self,
        f: &'a mut F,
    ) -> Box<
        dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn DerefMut<Target = F::Value> + 'b>)> + 'a,
    >
    where
        F: 'a,
    {
        match self {
            Either::Left(a) => a.values_mut(f),
            Either::Right(b) => b.values_mut(f),
        }
    }
}
impl<F: Func + ?Sized, A: HasChainableValues<F>, B: HasChainableValues<F>> HasChainableValues<F>
    for Either<A, B>
{
    fn values_chain<'a>(
        &'a self,
    ) -> Box<dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn Deref<Target = F::Value> + 'b>)> + 'a>
    {
        match self {
            Either::Left(a) => a.values_chain(),
            Either::Right(b) => b.values_chain(),
        }
    }

    fn values_chain_mut<'a>(
        &'a mut self,
    ) -> Box<
        dyn LendingIteratorDyn<Item = HKT!(<'b> => Box<dyn DerefMut<Target = F::Value> + 'b>)> + 'a,
    >
    where
        F: 'a,
    {
        match self {
            Either::Left(a) => a.values_chain_mut(),
            Either::Right(b) => b.values_chain_mut(),
        }
    }
}
pub trait Target<F: Func + ?Sized>: HasValues<F> + cfg_traits::Target<F> {
    fn push_value(&mut self, v: F::Value);
    fn from_values_and_block(a: impl Iterator<Item = F::Value>, k: F::Block) -> Self;
}
pub trait Term<F: Func + ?Sized>: HasValues<F> + cfg_traits::Term<F, Target: Target<F>> {}
impl<F: Func + ?Sized, T: HasValues<F> + cfg_traits::Term<F, Target: Target<F>>> Term<F> for T {}
