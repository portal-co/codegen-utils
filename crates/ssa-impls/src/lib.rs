// use std::collections::BTreeSet;
#![no_std]
extern crate alloc;
use alloc::vec::Vec;
use arena_traits::{Arena, IndexIter};
use cfg_traits::{Block, Target, Term};
use lending_iterator::lending_iterator::constructors::into_lending_iter;
use lending_iterator::prelude::{LendingIteratorDyn, HKT};
use lending_iterator::LendingIterator;
use ssa_traits::{Func, Target as SSATarget, TypedBlock, TypedFunc};
pub mod cfg;
pub mod dom;
pub mod maxssa;
pub mod reducify;
pub fn preds<F: cfg_traits::Func<Block: Clone + Eq>>(
    f: &F,
    k: F::Block,
) -> impl Iterator<Item = F::Block> + '_ {
    return f
        .blocks()
        .iter()
        .collect::<Vec<_>>()
        .into_iter()
        .filter(move |x| {
            let k = k.clone();
            f.blocks()[x.clone()]
                .term()
                .targets()
                .map::<HKT!(F::Block), _>(|[], a| a.block())
                .into_iter()
                .find(|c| *c == k)
                .is_some()
        });
}
pub fn add_phi<F: TypedFunc<Block: Clone + Eq, Value: Clone>>(
    f: &mut F,
    k: F::Block,
    ty: F::Ty,
    mut trial: impl FnMut(F::Block) -> F::Value,
) -> F::Value {
    let p = f.add_blockparam(k.clone(), ty);
    let i = f.blocks().iter().collect::<Vec<_>>();
    for k2 in i {
        let mut b = &mut f.blocks_mut()[k2.clone()];
        let mut i = b.term_mut().targets_mut();
        while let Some(mut target) = i.next() {
            if target.block() == k {
                target.push_value(trial(k2.clone()));
            }
        }
    }
    return p;
}
