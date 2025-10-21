use arena_traits::Arena;
use arena_traits::IndexIter;
use cfg_traits::Func;
use relooper::{RelooperLabel, ShapedBlock};
use ssa_impls::dom::dominates;
// use ssa_traits::Block;
use cfg_traits::Block;
// use ssa_traits::Term;
use cfg_traits::Target;
use cfg_traits::Term;
use lending_iterator::prelude::*;
// use waffle::{cfg::CFGInfo, Block, FunctionBody};
pub fn go<F: Func<Block: RelooperLabel>>(b: &F, k: F::Block) -> Box<ShapedBlock<F::Block>> {
    let cfg = ssa_impls::dom::domtree(b);
    // let reloop = std::panic::catch_unwind(|| {
    relooper::reloop(
        b.blocks()
            .iter()
            .filter(|k| dominates::<F>(&cfg, Some(b.entry()), Some(*k)))
            .map(|k| {
                let l = &b.blocks()[k];
                (
                    k,
                    l.term()
                        .targets()
                        .map::<HKT!(F::Block), _>(|[], a| a.block())
                        .into_iter()
                        .chain(
                            b.blocks()
                                .iter()
                                .filter(|x| dominates::<F>(&cfg, Some(*x), Some(k))),
                        )
                        .collect(),
                )
            })
            // .chain(once((Block::invalid(), vec![b.entry])))
            .collect(),
        // Block::invalid(),
        k,
    )
    // });
    // let reloop = match reloop {
    //     Ok(a) => a,
    //     Err(e) => {
    //         panic!(
    //             "reloop failure ({}) in {}",
    //             e.downcast_ref::<&str>()
    //                 .map(|a| *a)
    //                 .unwrap_or("unknown panic"),
    //             b.display("", None)
    //         );
    //     }
    // };
    // reloop
}
