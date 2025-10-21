/*
 * Derives from the dominator tree implementation in regalloc.rs, which is
 * licensed under the Apache Public License 2.0 with LLVM Exception. See:
 * https://github.com/bytecodealliance/regalloc.rs
 */
// This is an implementation of the algorithm described in
//
//   A Simple, Fast Dominance Algorithm
//   Keith D. Cooper, Timothy J. Harvey, and Ken Kennedy
//   Department of Computer Science, Rice University, Houston, Texas, USA
//   TR-06-33870
//   https://www.cs.rice.edu/~keith/EMBED/dom.pdf
use crate::preds;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use cfg_traits::Func;
type B<F> = Option<<F as Func>::Block>;
// pub type DoMap<O, T, Y, S> = BTreeMap<Block<O, T, Y, S>, JustBlock<O, T, Y, S>>;
// Helper
fn merge_sets<F: Func<Block: Ord + Clone>>(
    // map from Block to Block
    idom: &BTreeMap<B<F>, F::Block>,
    block_to_rpo: &BTreeMap<B<F>, u32>,
    mut node1: B<F>,
    mut node2: B<F>,
) -> B<F> {
    while node1 != node2 {
        if node1.is_none() || node2.is_none() {
            return None;
        }
        let rpo1 = block_to_rpo.get(&node1).copied();
        let rpo2 = block_to_rpo.get(&node2).copied();
        if rpo1 > rpo2 {
            node1 = Some(idom.get(&node1).unwrap().clone());
        } else if rpo2 > rpo1 {
            node2 = Some(idom.get(&node2).unwrap().clone());
        }
    }
    assert!(node1 == node2);
    node1
}
pub fn calculate<F: Func<Block: Ord + Clone>, PredFn: FnMut(B<F>) -> BTreeSet<B<F>>>(
    mut preds: PredFn,
    post_ord: &[B<F>],
    start: B<F>,
) -> BTreeMap<B<F>, F::Block> {
    // We have post_ord, which is the postorder sequence.
    // Compute maps from RPO to block number and vice-versa.
    let mut block_to_rpo: BTreeMap<B<F>, u32> = BTreeMap::default();
    for (i, rpo_block) in post_ord.iter().rev().enumerate() {
        block_to_rpo.insert(rpo_block.clone(), i as u32);
    }
    let mut idom: BTreeMap<B<F>, F::Block> = BTreeMap::default();
    // The start node must have itself as a parent.
    idom.insert(start.clone(), start.clone().unwrap());
    let mut changed = true;
    while changed {
        changed = false;
        // Consider blocks in reverse postorder. Skip any that are unreachable.
        for node in post_ord.iter().cloned().rev() {
            let rponum = *block_to_rpo.get(&node).unwrap();
            let mut parent = None;
            for pred in preds(node.clone()).iter().cloned() {
                let pred_rpo = match block_to_rpo.get(&pred).copied() {
                    Some(r) => r,
                    None => {
                        // Skip unreachable preds.
                        continue;
                    }
                };
                if pred_rpo < rponum {
                    parent = pred;
                    break;
                }
            }
            if parent.is_some() {
                for pred in preds(node.clone()).iter().cloned() {
                    if pred == parent {
                        continue;
                    }
                    if idom.get(&pred).cloned().is_none() {
                        continue;
                    }
                    parent = merge_sets::<F>(&idom, &block_to_rpo, parent, pred);
                }
            }
            if parent != idom.get(&node).cloned() {
                if let Some(parent) = parent {
                    idom.insert(node, parent);
                }
                changed = true;
            }
        }
    }
    // Now set the start node's dominator-tree parent to "invalid";
    // this allows the loop in `dominates` to terminate.
    idom.remove(&start);
    idom
}
pub fn domtree<F: Func<Block: Ord + Clone>>(f: &F) -> BTreeMap<B<F>, F::Block> {
    let rpo = crate::cfg::postorder(f)
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>();
    return calculate::<F, _>(
        |a| match a {
            Some(k) => preds(f, k).map(Some).collect(),
            None => BTreeSet::new(),
        },
        &rpo,
        Some(f.entry()),
    );
}
pub fn dominates<F: Func<Block: Ord + Clone>>(
    idom: &BTreeMap<B<F>, F::Block>,
    a: B<F>,
    mut b: B<F>,
) -> bool {
    loop {
        if a == b {
            return true;
        }
        if b.is_none() {
            return false;
        }
        b = idom.get(&b).cloned();
    }
}
