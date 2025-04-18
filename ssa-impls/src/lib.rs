// use std::collections::BTreeSet;
#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use arena_traits::{Arena, IndexIter};
use ssa_traits::{Func, TypedBlock, TypedFunc, Target as SSATarget};
use cfg_traits::{Block,Term,Target};
pub mod cfg;
pub mod dom;
pub mod maxssa;
pub mod reducify;
pub fn preds<F: cfg_traits::Func<Block: Clone + Eq>>(
    f: &F,
    k: F::Block,
) -> impl Iterator<Item = F::Block> + '_ {
    return f.blocks().iter().filter(move |x| {
        let k = k.clone();
        f.blocks()[x.clone()]
            .term()
            .targets()
            .find(|c| c.block() == k)
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
    for k2 in f.blocks().iter().collect::<Vec<_>>() {
        let mut b = &mut f.blocks_mut()[k2.clone()];
        for target in b.term_mut().targets_mut() {
            if target.block() == k {
                target.push_value(trial(k2.clone()));
            }
        }
    }
    return p;
}
