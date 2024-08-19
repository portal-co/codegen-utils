use std::ops::IndexMut;

pub trait Arena<Idx>: IndexMut<Idx>{
    fn alloc(&mut self, a: Self::Output) -> Idx;

}
#[cfg(feature = "id-arena")]
impl<T> Arena<id_arena::Id<T>> for id_arena::Arena<T>{
    fn alloc(&mut self, a: Self::Output) -> id_arena::Id<T> {
        self.alloc(a)
    }
}
#[macro_export]
macro_rules! simple_arena {
    ($idx:ty => $ty:ty as $id:ident) => {
        impl $crate::Arena<$idx> for $ty{
            fn alloc(&mut self, a: Self::Output) -> $idx{
                self.$id(a)
            }
        }
    };
}