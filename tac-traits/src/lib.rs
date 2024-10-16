#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use arena_traits::Arena;
use cfg_traits::BlockI;
use sift_trait::Sift;

pub trait Func:
    register_machine_traits::Func<Blocks: Arena<Self::Block, Output: Block<Self>>>
{
}
impl<
        T: register_machine_traits::Func<Blocks: Arena<Self::Block, Output: Block<Self>>> + ?Sized,
    > Func for T
{
}
pub trait Block<F: Func<Blocks: Arena<F::Block, Output = Self>> + ?Sized>:
    cfg_traits::Block<F>
{
    type Item;
    ///Exactly ONE register per LValue (optionally a store, or similar)
    type LValue: Sift<F::Reg, Residue: AsRef<F::Reg> + AsMut<F::Reg>>
        + AsRef<F::Reg>
        + AsMut<F::Reg>; 
    fn values<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a Self::LValue, &'a Self::Item)> + 'a>;
    fn values_mut<'a>(
        &'a mut self,
    ) -> Box<dyn Iterator<Item = (&'a mut Self::LValue, &'a mut Self::Item)> + 'a>;
    fn add_value(&mut self, target: Self::LValue, item: Self::Item);
}

pub type ItemI<F> = <BlockI<F> as Block<F>>::Item;
pub type LValueI<F> = <BlockI<F> as Block<F>>::LValue;
pub type LValueNonRegI<F> =
    <LValueI<F> as Sift<<F as register_machine_traits::Func>::Reg>>::Residue;
