use arena_traits::Arena;
use ssa_traits::{Block, HasValues, Target, Term, TypedBlock, TypedFunc};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    hash::Hash,
    ops::Index,
};
pub trait RedFunc:
    TypedFunc<
    Block: Ord + Hash + Clone,
    Value: Hash + Clone + Ord,
    Values: Arena<Self::Value, Output: Clone>,
    Blocks: Arena<Self::Block, Output: Block<Self, Terminator: Clone> + Default>,
>
{
}
pub struct Reducifier<F: RedFunc> {
    blocks: BTreeMap<F::Block, BlockState<F>>,
}

impl<F: RedFunc> Default for Reducifier<F>{
    fn default() -> Self {
        Self { blocks: Default::default() }
    }
}

struct BlockState<F: RedFunc> {
    headers: BTreeSet<F::Block>,
    is_header: bool,
}
impl<F: RedFunc> Default for BlockState<F> {
    fn default() -> Self {
        Self {
            headers: Default::default(),
            is_header: Default::default(),
        }
    }
}
impl<F: RedFunc> Clone for BlockState<F> {
    fn clone(&self) -> Self {
        Self {
            headers: self.headers.clone(),
            is_header: self.is_header.clone(),
        }
    }
}

impl<F: RedFunc> Reducifier<F> {
    // pub fn new(body: &'a FunctionBody) -> Reducifier<'a> {
    //     let cfg = CFGInfo::new(body);
    //     Reducifier {
    //         body,
    //         cfg,
    //         blocks: PerEntity::default(),
    //     }
    // }

    pub fn run(&mut self, body: &mut F) {
        // First, compute all of the loop header-sets.
        // - Start by computing RPO.
        // - Find backedges (edges (a, b) where rpo(b) <= rpo(a)).
        // - For each backedge, mark extent of rpo-indices "under"
        //   edge as within header.
        // - Do one forward pass to properly nest regions, keeping
        //   stack of headers when we entered their regions and
        //   enforcing LIFO by extending appropriately.
        let mut rpo_ = crate::cfg::postorder(body);
        rpo_.reverse();

        for (rpo, block) in rpo_.iter().cloned().enumerate() {
            for succ in body.blocks()[block]
                .term()
                .targets()
                .map(|a| a.block())
                .collect::<BTreeSet<_>>()
            {
                let succ_rpo = rpo_
                    .iter()
                    .enumerate()
                    .find_map(|(a, b)| if *b == succ { Some(a) } else { None })
                    .unwrap();
                if succ_rpo <= rpo {
                    for i in succ_rpo..=rpo {
                        let b = rpo_[i].clone();
                        self.blocks
                            .entry(b.clone())
                            .or_insert_with(Default::default)
                            .headers
                            .insert(succ.clone());
                        self.blocks
                            .entry(b)
                            .or_insert_with(Default::default)
                            .is_header = true;
                    }
                }
            }
        }

        let mut header_stack: Vec<F::Block> = vec![];
        for block in rpo_.iter() {
            while let Some(innermost) = header_stack.last() {
                if !self
                    .blocks
                    .entry(block.clone())
                    .or_insert_with(Default::default)
                    .headers
                    .contains(innermost)
                {
                    header_stack.pop();
                } else {
                    break;
                }
            }
            if self
                .blocks
                .entry(block.clone())
                .or_insert_with(Default::default)
                .is_header
            {
                header_stack.push(block.clone());
            }

            for header in &header_stack {
                self.blocks
                    .entry(block.clone())
                    .or_insert_with(Default::default)
                    .headers
                    .insert(header.clone());
            }
        }

        // Now, check whether any irreducible edges exist: edges from
        // B1 to B2 where headers(B2) - headers(B1) - {B2} is not
        // empty (i.e., the edge jumps into a new loop -- adds a new
        // header -- without going through that header block).
        let mut irreducible_headers: BTreeSet<F::Block> = Default::default();
        for (block, data) in body.blocks().iter().map(|a| (a.clone(), &body.blocks()[a])) {
            let headers = &self
                .blocks
                .entry(block.clone())
                .or_insert_with(Default::default)
                .headers.clone();
            for succ in &data
                .term()
                .targets()
                .map(|a| a.block())
                .collect::<BTreeSet<_>>()
            {
                // eprintln!("examining edge {} -> {}", block, succ);
                for succ_header in &self
                    .blocks
                    .entry(succ.clone())
                    .or_insert_with(Default::default)
                    .headers
                {
                    // eprintln!("  successor {} has header {}", succ, succ_header);
                    if succ_header != succ && !headers.contains(succ_header) {
                        eprintln!("    -> irreducible edge");
                        irreducible_headers.insert(succ_header.clone());
                    }
                }
            }
        }

        if irreducible_headers.is_empty() {
            return;
        }

        // if log::log_enabled!(log::Level::Trace) {
        //     for block in self.body.blocks.iter() {
        //         let mut headers = self.blocks[block]
        //             .headers
        //             .iter()
        //             .cloned()
        //             .collect::<Vec<_>>();
        //         headers.sort();
        //         log::trace!("* {}: header set {:?}", block, headers);
        //     }
        // }

        // Now, in the irreducible case, "elaborate" the CFG.

        // First do limited conversion to max-SSA to fix up references
        // across contexts.
        // let mut cut_blocks = HashSet::default();
        // for (block, data) in body.blocks().iter().map(|a|(a.clone(),&body.blocks()[a]))  {
        //     for &succ in &data.succs {
        //         // Loop exits
        //         for header in &self.blocks[block].headers {
        //             if !self.blocks[succ].headers.contains(header)
        //                 && irreducible_headers.contains(header)
        //             {
        //                 log::trace!("cut-block at loop exit: {}", succ);
        //                 cut_blocks.insert(succ);
        //             }
        //         }
        //         // Loop side entries
        //         for header in &self.blocks[succ].headers {
        //             if !self.blocks[block].headers.contains(header) && *header != succ {
        //                 log::trace!("cut-block at loop side entry: {}", succ);
        //                 cut_blocks.insert(succ);
        //             }
        //         }
        //     }
        // }

        let mut new_body = body;
        crate::maxssa::maxssa(new_body);
        // let cfg = CFGInfo::new(&new_body);
        // crate::passes::maxssa::run(&mut new_body, Some(cut_blocks), &cfg);
        // crate::passes::resolve_aliases::run(&mut new_body);

        // log::trace!("after max-SSA run:\n{}\n", new_body.display("| ", None));

        // Implicitly, context {} has an identity-map from old block
        // number to new block number. We use the map only for
        // non-empty contexts.
        let mut context_map: HashMap<Vec<F::Block>, usize> = HashMap::default();
        let mut contexts: Vec<Vec<F::Block>> = vec![vec![]];
        context_map.insert(vec![], 0);
        let mut block_map: HashMap<(usize, F::Block), F::Block> = HashMap::default();
        let mut value_map: HashMap<(usize, F::Value), F::Value> = HashMap::default();

        // List of (ctx, new block) tuples for duplicated code.
        let mut cloned_blocks: Vec<(usize, F::Block)> = vec![];
        // Map from block in new body to (ctx, orig block) target, to
        // allow updating terminators.
        let mut terminators: HashMap<F::Block, Vec<(usize, F::Block)>> = HashMap::default();

        let mut queue: VecDeque<(usize, F::Block)> = VecDeque::new();
        let mut visited: HashSet<(usize, F::Block)> = HashSet::default();
        queue.push_back((0, new_body.entry()));
        visited.insert((0, new_body.entry()));
        while let Some((ctx, block)) = queue.pop_front() {
            // log::trace!(
            //     "elaborate: block {} in context {} ({:?})",
            //     block,
            //     ctx,
            //     contexts[ctx]
            // );

            // If this is a non-default context, replicate the block.
            let new_block = if ctx != 0 {
                // log::trace!("cloning block {} in new context", block);
                let new_block = new_body.blocks_mut().alloc(Default::default());
                // new_body.blocks[new_block].desc = format!("Cloned {}", block);
                let params = new_body.blocks()[block.clone()]
                    .params()
                    .collect::<Vec<_>>();
                for (ty, val) in params {
                    let blockparam = new_body.add_blockparam(new_block.clone(), ty);
                    value_map.insert((ctx, val), blockparam);
                }

                block_map.insert((ctx, block.clone()), new_block.clone());
                cloned_blocks.push((ctx, new_block.clone()));

                // Copy over all value definitions, but don't rewrite
                // args yet -- we'll do a separate pass for that.
                let insts = new_body.blocks()[block.clone()].insts().collect::<Vec<_>>();
                for value in insts {
                    let def = new_body.values()[value.clone()].clone();
                    let new_value = new_body.values_mut().alloc(def);
                    value_map.insert((ctx, value.clone()), new_value.clone());
                    <<F::Blocks as Index<F::Block>>::Output as Block<F>>::add_inst(
                        new_body,
                        new_block.clone(),
                        new_value,
                    );
                }

                // Copy over the terminator but don't update yet --
                // we'll do that later too.
                *new_body.blocks_mut()[new_block.clone()].term_mut() =
                    new_body.blocks()[block.clone()].term().clone();

                new_block
            } else {
                block.clone()
            };

            // For every terminator, determine the target context:
            //
            // let ToContext = headers(To) & !{To} & (FromContext U !headers(From))
            let term = terminators.entry(new_block.clone()).or_insert_with(|| vec![]);
            let succs = new_body.blocks()[block.clone()]
                .term()
                .targets()
                .map(|a| a.block())
                .collect::<BTreeSet<_>>();
            for succ in succs {
                let mut ctx_blocks = self
                    .blocks
                    .entry(succ.clone())
                    .or_insert_with(Default::default)
                    .headers
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>();
                ctx_blocks.sort();
                ctx_blocks.retain(|header_block| {
                    header_block != &succ
                        && (contexts[ctx].contains(&header_block)
                            || !self
                                .blocks
                                .entry(block.clone())
                                .or_insert_with(Default::default)
                                .headers
                                .contains(&header_block))
                });
                let to_ctx = *context_map.entry(ctx_blocks.clone()).or_insert_with(|| {
                    let id = contexts.len();
                    contexts.push(ctx_blocks);
                    id
                });
                // log::trace!(
                //     "elaborate: edge {} -> {} from ctx {:?} goes to ctx {:?}",
                //     block,
                //     succ,
                //     contexts[ctx],
                //     contexts[to_ctx]
                // );

                term.push((to_ctx, succ.clone()));

                if visited.insert((to_ctx, succ.clone())) {
                    // log::trace!("enqueue block {} ctx {}", succ, to_ctx);
                    queue.push_back((to_ctx, succ));
                }
            }
        }

        // Second pass: rewrite args, and set up terminators. Both
        // happen in a second pass so that we have the block- and
        // value-map available for all blocks and values, regardless
        // of cycles or processing order.
        for (ctx, new_block) in cloned_blocks {
            for inst in &new_body.blocks()[new_block.clone()].insts().collect::<Vec<_>>() {
                let mut v = new_body.values_mut()[inst.clone()].clone();
                for val in v.values_mut(new_body) {
                    *val = value_map
                        .get(&(ctx, val.clone()))
                        .cloned()
                        .unwrap_or(val.clone());
                }
                new_body.values_mut()[inst.clone()] = v;
            }
            let mut t = new_body.blocks_mut()[new_block.clone()]
            .term_mut().clone();
            for val in t
                .values_mut(new_body)
            {
                *val = value_map
                    .get(&(ctx, val.clone()))
                    .cloned()
                    .unwrap_or(val.clone());
            }
            *new_body.blocks_mut()[new_block.clone()]
            .term_mut() = t;
        }

        for block in new_body.blocks().iter().collect::<Vec<_>>().into_iter() {
            // log::trace!("processing terminators for block {}", block);
            let block_def =&mut new_body.blocks_mut()[block.clone()];
            let terms = match terminators.get(&block) {
                Some(t) => t,
                // If no entry in `terminators`, we didn't visit the
                // block; it must not be reachable.
                None => continue,
            };
            let mut terms = terms.iter();
            for target in block_def.term_mut().targets_mut(){
                let (to_ctx, to_orig_block) = terms.next().unwrap().clone();
                *target.block_mut() = block_map
                    .get(&(to_ctx, to_orig_block.clone()))
                    .cloned()
                    .unwrap_or(to_orig_block);
            }
        }

        // new_body.recompute_edges();

        // log::trace!("After duplication:\n{}\n", new_body.display("| ", None));

        // new_body.validate().unwrap();
        // new_body.verify_reducible().unwrap();

        // Cow::Owned(new_body)
    }
}
