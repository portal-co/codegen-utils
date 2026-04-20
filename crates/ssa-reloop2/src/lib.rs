use std::collections::{BTreeMap, BTreeSet};

use arena_traits::IndexIter;
use cfg_traits::{Block as CfgBlock, Func, Target, Term};
use ssa_impls::{
    cfg::postorder,
    dom::{dominates, domtree},
};

pub type LoopId = u32;

#[derive(Debug, Clone)]
pub enum BranchMode {
    LoopBreak(LoopId),
    LoopBreakIntoMulti(LoopId),
    LoopContinue(LoopId),
    LoopContinueIntoMulti(LoopId),
    MergedBranch,
    MergedBranchIntoMulti,
    SetLabelAndBreak,
}

pub struct SimpleBlock<L: Ord + Clone> {
    pub label: L,
    pub immediate: Option<Box<StructuredBlock<L>>>,
    pub branches: BTreeMap<L, BranchMode>,
    pub next: Option<Box<StructuredBlock<L>>>,
}

pub struct LoopBlock<L: Ord + Clone> {
    pub loop_id: LoopId,
    pub inner: Box<StructuredBlock<L>>,
    pub next: Option<Box<StructuredBlock<L>>>,
}

pub struct MultipleBlock<L: Ord + Clone> {
    pub handled: Vec<HandledBlock<L>>,
}

pub struct HandledBlock<L: Ord + Clone> {
    pub labels: Vec<L>,
    pub inner: StructuredBlock<L>,
    pub break_after: bool,
}

pub enum StructuredBlock<L: Ord + Clone> {
    Simple(SimpleBlock<L>),
    Loop(LoopBlock<L>),
    Multiple(MultipleBlock<L>),
}

/// Stackify a CFG into structured control flow.
///
/// Generic over any `cfg_traits::Func` — does not depend on the `relooper` crate.
pub fn go<F>(f: &F) -> Box<StructuredBlock<F::Block>>
where
    F: Func,
    F::Block: Ord + Clone,
{
    eprintln!("[ssa-reloop2] go called");
    let mut rpo = postorder(f);
    eprintln!("[ssa-reloop2] postorder done: {} blocks", rpo.len());
    rpo.reverse(); // entry is rpo[0]

    assert!(!rpo.is_empty(), "CFG has no reachable blocks");

    let idom = domtree(f);
    eprintln!("[ssa-reloop2] domtree done");

    // Predecessor map for efficient loop-body computation.
    let mut pred_map: BTreeMap<F::Block, Vec<F::Block>> = BTreeMap::new();
    for a in f.blocks().iter() {
        for t in f.blocks()[a.clone()].term().targets() {
            pred_map.entry(t.block()).or_default().push(a.clone());
        }
    }
    eprintln!("[ssa-reloop2] pred_map done");

    // Back edges: (a→b) is a back edge iff b dominates a.
    let mut back_edges: BTreeSet<(F::Block, F::Block)> = BTreeSet::new();
    for a in f.blocks().iter() {
        for t in f.blocks()[a.clone()].term().targets() {
            let b = t.block();
            if dominates::<F>(&idom, Some(b.clone()), Some(a.clone())) {
                back_edges.insert((a.clone(), b));
            }
        }
    }

    let loop_headers: BTreeSet<F::Block> =
        back_edges.iter().map(|(_, h)| h.clone()).collect();

    // Loop body of header H: all blocks dominated by H that can reach H via a back edge.
    let mut loop_body: BTreeMap<F::Block, BTreeSet<F::Block>> = BTreeMap::new();
    for header in &loop_headers {
        let mut body: BTreeSet<F::Block> = BTreeSet::new();
        body.insert(header.clone());
        let mut worklist: Vec<F::Block> = back_edges
            .iter()
            .filter(|(_, h)| h == header)
            .map(|(latch, _)| latch.clone())
            .collect();
        while let Some(b) = worklist.pop() {
            if body.contains(&b) {
                continue;
            }
            if dominates::<F>(&idom, Some(header.clone()), Some(b.clone())) {
                body.insert(b.clone());
                if let Some(preds) = pred_map.get(&b) {
                    for p in preds {
                        worklist.push(p.clone());
                    }
                }
            }
        }
        loop_body.insert(header.clone(), body);
    }

    eprintln!("[ssa-reloop2] rpo={} back_edges={} headers={}", rpo.len(), back_edges.len(), loop_headers.len());

    let mut next_id: u32 = 0;
    build::<F>(
        &rpo,
        &idom,
        &back_edges,
        &loop_headers,
        &loop_body,
        &[],
        &mut next_id,
        f,
    )
    .expect("non-empty RPO produces a structured block")
}

fn successors<F: Func<Block: Clone>>(f: &F, b: F::Block) -> Vec<F::Block> {
    f.blocks()[b].term().targets().map(|t| t.block()).collect()
}

/// Recursive Stackifier: processes `rpo_slice` in order, emitting structured blocks.
fn build<F: Func<Block: Ord + Clone>>(
    rpo_slice: &[F::Block],
    idom: &BTreeMap<Option<F::Block>, F::Block>,
    back_edges: &BTreeSet<(F::Block, F::Block)>,
    loop_headers: &BTreeSet<F::Block>,
    loop_body: &BTreeMap<F::Block, BTreeSet<F::Block>>,
    loop_stack: &[(F::Block, u32)], // innermost first
    next_id: &mut u32,
    f: &F,
) -> Option<Box<StructuredBlock<F::Block>>> {
    if rpo_slice.is_empty() {
        return None;
    }

    let b = rpo_slice[0].clone();
    let slice_set: BTreeSet<F::Block> = rpo_slice.iter().cloned().collect();
    let already_in_stack = loop_stack.iter().any(|(h, _)| *h == b);

    // ── Loop header: wrap the body in a Loop node ──────────────────────────
    if loop_headers.contains(&b) && !already_in_stack {
        let loop_id = *next_id;
        *next_id += 1;

        let body = loop_body.get(&b).expect("header has loop body");
        // Last index in rpo_slice that belongs to the loop body.
        let body_end = rpo_slice
            .iter()
            .rposition(|blk| body.contains(blk))
            .unwrap_or(0);

        let body_slice = &rpo_slice[..=body_end];
        let rest = &rpo_slice[body_end + 1..];

        let mut new_stack: Vec<(F::Block, u32)> = Vec::with_capacity(loop_stack.len() + 1);
        new_stack.push((b.clone(), loop_id));
        new_stack.extend_from_slice(loop_stack);

        let inner =
            build::<F>(body_slice, idom, back_edges, loop_headers, loop_body, &new_stack, next_id, f)
                .expect("loop body slice is non-empty");
        let next =
            build::<F>(rest, idom, back_edges, loop_headers, loop_body, loop_stack, next_id, f);

        return Some(Box::new(StructuredBlock::Loop(LoopBlock {
            loop_id,
            inner,
            next,
        })));
    }

    // ── Plain block ────────────────────────────────────────────────────────
    let succs = successors::<F>(f, b.clone());
    let mut branches: BTreeMap<F::Block, BranchMode> = BTreeMap::new();
    let mut fwd_in_slice: Vec<F::Block> = Vec::new();

    for s in &succs {
        if back_edges.contains(&(b.clone(), s.clone())) {
            // Back edge → LoopContinue to the matching loop.
            let id = loop_stack
                .iter()
                .find(|(h, _)| h == s)
                .map(|(_, id)| *id)
                .unwrap_or(0);
            branches.insert(s.clone(), BranchMode::LoopContinue(id));
        } else if !slice_set.contains(s) {
            // Exits the current scope.
            if let Some((_, id)) = loop_stack.first() {
                // Inside a loop: emit a labeled break to the innermost loop.
                branches.insert(s.clone(), BranchMode::LoopBreak(*id));
            } else {
                // Outside all loops: the successor will be handled by the
                // enclosing Multiple's `next`. Just set CFF and fall through.
                branches.insert(s.clone(), BranchMode::MergedBranch);
            }
        } else {
            branches.insert(s.clone(), BranchMode::MergedBranch);
            fwd_in_slice.push(s.clone());
        }
    }

    // Sort forward in-slice successors by their position in this slice.
    let pos: BTreeMap<F::Block, usize> =
        rpo_slice.iter().enumerate().map(|(i, b)| (b.clone(), i)).collect();
    fwd_in_slice.sort_by_key(|b| pos[b]);

    // Simple fallthrough: zero or one forward successor that is the next block.
    let is_fallthrough = fwd_in_slice.len() == 0
        || (fwd_in_slice.len() == 1 && rpo_slice.get(1) == fwd_in_slice.first());

    if is_fallthrough {
        let next =
            build::<F>(&rpo_slice[1..], idom, back_edges, loop_headers, loop_body, loop_stack, next_id, f);
        return Some(Box::new(StructuredBlock::Simple(SimpleBlock {
            label: b,
            branches,
            immediate: None,
            next,
        })));
    }

    // Conditional / multi-way branch: emit a Multiple block.
    let after = &rpo_slice[1..];
    let r = find_reconverge::<F>(after, &fwd_in_slice, idom);
    // after[..r] = branch regions, after[r..] = tail (reconverge + continuation)
    let branch_region = &after[..r];
    let tail = &after[r..];

    let handled = partition_branches::<F>(
        branch_region,
        &fwd_in_slice,
        idom,
        back_edges,
        loop_headers,
        loop_body,
        loop_stack,
        next_id,
        f,
    );

    let next =
        build::<F>(tail, idom, back_edges, loop_headers, loop_body, loop_stack, next_id, f);

    Some(Box::new(StructuredBlock::Simple(SimpleBlock {
        label: b,
        branches,
        immediate: Some(Box::new(StructuredBlock::Multiple(MultipleBlock { handled }))),
        next,
    })))
}

/// Returns the index R in `after` where the branches reconverge.
///
/// `after[..R]` is the union of all branch regions; `after[R..]` is the tail
/// that runs unconditionally after the Multiple block.
///
/// Rule:
///   * A direct successor (fwd_succ entry) always belongs to some branch region —
///     skip it during the scan.
///   * A non-entry block belongs to the branch region iff at least one fwd_succ
///     *strictly* dominates it (i.e., the only path to it goes through that fwd_succ).
///   * The reconverge is the first block that neither is a direct successor nor is
///     strictly dominated by any single fwd_succ.
///   * If all non-entry blocks are owned (e.g., a loop body dominates its own exit),
///     fall back to treating the *last* direct successor in RPO order as the
///     reconverge — its branch region is empty, so it runs unconditionally after
///     the others.
fn find_reconverge<F: Func<Block: Ord + Clone>>(
    after: &[F::Block],
    fwd_succs: &[F::Block],
    idom: &BTreeMap<Option<F::Block>, F::Block>,
) -> usize {
    let fwd_set: BTreeSet<F::Block> = fwd_succs.iter().cloned().collect();

    // Phase 1: first non-entry block not strictly dominated by any entry.
    for (i, b) in after.iter().enumerate() {
        if fwd_set.contains(b) {
            continue; // entries belong to their own branch, skip for reconverge scan
        }
        let strictly_owned = fwd_succs
            .iter()
            .any(|s| s != b && dominates::<F>(idom, Some(s.clone()), Some(b.clone())));
        if !strictly_owned {
            return i;
        }
    }

    // Phase 2: every non-entry block is strictly dominated by some entry (e.g.
    // all blocks between entry and a back-edge form a dominated chain).  The
    // last direct-successor entry has an empty branch region and acts as the
    // reconverge point.
    if let Some(last) = after.iter().rposition(|b| fwd_set.contains(b)) {
        return last;
    }

    after.len()
}

/// Assigns each block in `branch_region` to the fwd_succ that dominates it,
/// then recursively builds each handled block.
fn partition_branches<F: Func<Block: Ord + Clone>>(
    branch_region: &[F::Block],
    fwd_succs: &[F::Block], // sorted by RPO position
    idom: &BTreeMap<Option<F::Block>, F::Block>,
    back_edges: &BTreeSet<(F::Block, F::Block)>,
    loop_headers: &BTreeSet<F::Block>,
    loop_body: &BTreeMap<F::Block, BTreeSet<F::Block>>,
    loop_stack: &[(F::Block, u32)],
    next_id: &mut u32,
    f: &F,
) -> Vec<HandledBlock<F::Block>> {
    fwd_succs
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            // Region for s: from s's position in branch_region to the next succ's position (or end).
            let start = branch_region.iter().position(|x| x == s).unwrap_or(branch_region.len());
            let end = fwd_succs
                .get(idx + 1)
                .and_then(|next_s| branch_region.iter().position(|x| x == next_s))
                .unwrap_or(branch_region.len());

            let region = &branch_region[start..end];
            let inner = build::<F>(region, idom, back_edges, loop_headers, loop_body, loop_stack, next_id, f)
                .unwrap_or_else(|| {
                    // Empty region — emit a bare Simple with no body.
                    // This happens when the branch jumps directly to the reconverge point.
                    Box::new(StructuredBlock::Simple(SimpleBlock {
                        label: s.clone(),
                        branches: BTreeMap::new(),
                        immediate: None,
                        next: None,
                    }))
                });

            HandledBlock {
                labels: vec![s.clone()],
                inner: *inner,
                break_after: true,
            }
        })
        .collect()
}
