#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use arena_traits::Arena;
use cfg_traits::BlockI;

pub trait Func: register_machine_traits::Func<Blocks: Arena<Self::Block,Output: Block<Self>>>{

}
impl<T: register_machine_traits::Func<Blocks: Arena<Self::Block,Output: Block<Self>>> + ?Sized> Func for T{

}
pub trait Block<F: Func<Blocks: Arena<F::Block,Output = Self>> + ?Sized>: cfg_traits::Block<F>{
    type Item;
    fn values<'a>(&'a self) -> Box<dyn Iterator<Item = (F::Reg,&'a Self::Item)> + 'a>;
    fn values_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = (&'a mut F::Reg,&'a mut Self::Item)> + 'a>;
    fn add_value(&mut self, target: F::Reg, item: Self::Item);
}
pub type ItemI<F> = <BlockI<F> as Block<F>>::Item;