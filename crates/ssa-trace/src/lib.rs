#![no_std]
extern crate alloc;
use alloc::collections::BTreeMap;
use cfg_traits::TargetI;
use valser::ValSer;
pub trait Trace<F: cfg_traits::Func, G: ssa_traits::Func>: Sized {
    type State: ValSer<G::Value>;
    type Instance;
    fn run(
        &mut self,
        f: &F,
        g: &mut G,
        i: Self::Instance,
        k: F::Block,
        tracer: &mut Tracer<F, G, Self>,
    ) -> anyhow::Result<Self::State>;
    fn transfer(
        &mut self,
        f: &F,
        g: &mut G,
        i: &Self::State,
        k: F::Block,
        t: &TargetI<F>,
        tracer: &mut Tracer<F, G, Self>,
    ) -> anyhow::Result<Self::Instance>;
}
pub struct Tracer<F: cfg_traits::Func, G: ssa_traits::Func, T: Trace<F, G>> {
    pub wrapped: T,
    pub all: BTreeMap<(F::Block, T::Instance), G::Block>,
}
