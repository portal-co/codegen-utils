use std::collections::{BTreeMap, BTreeSet};

use ssa_traits::{Block, Func, Target, Term};

pub fn calculate_postorder<
    F: Func<Block: Ord + Clone>,
    SuccFn: FnMut(F::Block) -> Vec<F::Block>,
>(
    entry: F::Block,
    mut succ_blocks: SuccFn,
) -> Vec<F::Block> {
    let mut ret = vec![];

    // State: visited-block map, and explicit DFS stack.
    let mut visited: BTreeSet<F::Block> = BTreeSet::new();

    #[derive(Debug)]
    struct State<F: Func> {
        block: F::Block,
        succs: Vec<F::Block>,
        next_succ: usize,
    }
    let mut stack: Vec<State<F>> = vec![];

    visited.insert(entry.clone());
    stack.push(State {
        block: entry.clone(),
        succs: succ_blocks(entry.clone()),
        next_succ: 0,
    });

    while let Some(ref mut state) = stack.last_mut() {
        // log::trace!("postorder: TOS is {:?}", state);
        // Perform one action: push to new succ, skip an already-visited succ, or pop.
        if state.next_succ < state.succs.len() {
            let succ = state.succs[state.next_succ].clone();
            // log::trace!(" -> succ {}", succ);
            state.next_succ += 1;
            if !visited.contains(&succ) {
                // log::trace!(" -> visiting");
                visited.insert(succ.clone());
                stack.push(State {
                    block: succ.clone(),
                    succs: succ_blocks(succ.clone()),
                    next_succ: 0,
                });
            }
        } else {
            // log::trace!("retreating from {}", state.block);
            ret.push(state.block.clone());
            stack.pop();
        }
    }

    ret
}
pub fn postorder<F: Func<Block: Ord + Clone>>(f: &F) -> Vec<F::Block> {
    return calculate_postorder::<F, _>(f.entry(), |a| {
        f.blocks()[a].term().targets().map(|a| a.block()).collect()
    });
}
