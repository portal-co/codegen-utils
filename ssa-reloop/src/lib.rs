use arena_traits::Arena;
use relooper::{RelooperLabel, ShapedBlock};
use ssa_impls::dom::dominates;
use ssa_traits::Func;
use ssa_traits::Block;
use ssa_traits::Term;
use ssa_traits::Target;
// use waffle::{cfg::CFGInfo, Block, FunctionBody};

pub fn go<F: Func<Block: RelooperLabel>>(b: &F) -> Box<ShapedBlock<F::Block>> {
    let cfg = ssa_impls::dom::domtree(b);
    // let reloop = std::panic::catch_unwind(|| {
        relooper::reloop(
            b.blocks().iter()
                .filter(|k| dominates::<F>(&cfg,Some(b.entry()), Some(*k)))
                .map(|k| {
                    let l = &b.blocks()[k];
                    (
                        k,
                        l.term().targets()
                            .map(|a|a.block())
                            .chain(b.blocks().iter().filter(|x| dominates::<F>(&cfg,Some(*x), Some(k))))
                            .collect(),
                    )
                })
                // .chain(once((Block::invalid(), vec![b.entry])))
                .collect(),
            // Block::invalid(),
            b.entry(),
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
