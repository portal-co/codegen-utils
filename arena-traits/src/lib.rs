#![no_std]
extern crate alloc;
use core::ops::IndexMut;

use alloc::boxed::Box;

pub trait IndexAlloc<Idx>: IndexMut<Idx> {
    fn alloc(&mut self, a: Self::Output) -> Idx;
}
pub trait IndexIter<Idx>: IndexMut<Idx> {
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = Idx> + 'a>;
}
pub trait Arena<Idx>: IndexAlloc<Idx> + IndexIter<Idx> {}
impl<Idx, T: IndexAlloc<Idx> + IndexIter<Idx>> Arena<Idx> for T {}
#[cfg(feature = "id-arena")]
const _: () = {
    impl<T> IndexAlloc<id_arena::Id<T>> for id_arena::Arena<T> {
        fn alloc(&mut self, a: Self::Output) -> id_arena::Id<T> {
            self.alloc(a)
        }
    };
    impl<T> IndexIter<id_arena::Id<T>> for id_arena::Arena<T> {
        fn iter<'a>(&'a self) -> Box<(dyn Iterator<Item = id_arena::Id<T>> + 'a)> {
            Box::new(self.iter().map(|a| a.0))
        }
    }
};
impl<T> IndexIter<usize> for [T] {
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = usize> + 'a> {
        Box::new(self.iter().enumerate().map(|a| a.0))
    }
}
impl<T, const N: usize> IndexIter<usize> for [T; N] {
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = usize> + 'a> {
        Box::new(0..N)
    }
}
