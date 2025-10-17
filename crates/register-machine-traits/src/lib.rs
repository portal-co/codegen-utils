#![no_std]

use core::ops::IndexMut;

use arena_traits::IndexIter;
extern crate alloc;

pub trait Func: cfg_traits::Func{
    type Reg;
    type Regs: IndexIter<Self::Reg>;
    fn regs(&self) -> &Self::Regs;
    fn regs_mut(&mut self) -> &mut Self::Regs;
}