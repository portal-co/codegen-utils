pub mod to_ssa_traits;
#[doc(hidden)]
pub mod __rexport{
    pub use pliron;
    pub use linkme;
}
#[macro_export]
macro_rules! pliron_compat_op {
    ($f:ty => $a:ident) => {
        pub trait $a: $crate::to_ssa_traits::PlironCompatOp<$f>{

        }
        impl<T: $crate::to_ssa_traits::PlironCompatOp<$f>> $a for T{

        }
        const _: () = {
            #[$crate::linkme::distributed_slice($crate::pliron::op::OP_INTERFACE_DEPS)]
            static INTERFACE_DEP: std::sync::LazyLock<(std::any::TypeId, Vec<std::any::TypeId>)>
                = std::sync::LazyLock::new(|| {
                    (std::any::TypeId::of::<dyn $a>(), vec![])
             });
        };
    };
}
