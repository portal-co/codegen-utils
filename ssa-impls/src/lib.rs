use std::collections::BTreeSet;

use arena_traits::Arena;
use ssa_traits::{Block, Func, Target, Term};
pub mod dom;
pub mod cfg;
pub mod maxssa;
pub fn preds<F: Func<Block: Clone + Eq>>(f: &F, k: F::Block) -> impl Iterator<Item = F::Block> + '_{
    return f.blocks().iter().filter(move|x| {
        let k = k.clone();
        f.blocks()[x.clone()]
            .term()
            .targets()
            .find(|c| c.block() == k)
            .is_some()
    });
}
