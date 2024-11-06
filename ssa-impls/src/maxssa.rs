use alloc::{collections::{BTreeMap, BTreeSet}, vec::Vec};

use arena_traits::{Arena, IndexIter};
use ssa_traits::{Block, Func, HasValues, Target, Term, TypedFunc, TypedValue, Value};
use cfg_traits::{Block as CFGBlock, Func as CFGFunc, Target as CFGTarget, Term as CFGTerm};
use core::hash::Hash;
use lending_iterator::prelude::*;

use crate::preds;

// use id_arena::Id;

pub fn maxssa<
    F: TypedFunc<
        Block: Ord + Hash + Clone,
        Value: Hash + Clone + Ord,
        Values: Arena<F::Value, Output: Clone>,
        Blocks: Arena<F::Block, Output: Block<F, Terminator: Clone>>,
    >,
>(
    f: &mut F,
) {
    MaxSSAPass::new().run(f);
}

// use crate::{util::PerID, Block, Func, SaneTerminator, Use, Value};

struct MaxSSAPass<F: Func<Block: Ord + Hash + Clone, Value: Hash + Clone>> {
    /// Additional block args that must be passed to each block, in
    /// order. Value numbers are *original* values.
    new_args: BTreeMap<F::Block, Vec<F::Value>>,
    /// For each block, a value map: from original value to local copy
    /// of value.
    value_map: BTreeMap<(F::Block, F::Value), F::Value>,
}

impl<
        F: TypedFunc<
            Block: Ord + Hash + Clone,
            Value: Hash + Clone + Ord,
            Values: Arena<F::Value, Output: Clone>,
            Blocks: Arena<F::Block, Output: Block<F, Terminator: Clone>>,
        >,
    > MaxSSAPass<F>
{
    fn new() -> Self {
        Self {
            new_args: BTreeMap::new(),
            value_map: BTreeMap::new(),
        }
    }

    fn run(mut self, body: &mut F) {
        let ks = body.blocks().iter().collect::<Vec<_>>();
        for block in ks {
            self.visit(body, block);
        }
        // eprintln!("{:?}",self.new_args.data.iter().enumerate().map(|(a,b)|(a,b.iter().map(|a|a.value.index()).collect::<Vec<_>>())).collect::<Vec<_>>());
        self.update(body);
    }

    fn visit(&mut self, body: &mut F, block: F::Block) {
        // For each use in the block, process the use. Collect all
        // uses first to deduplicate and allow more efficient
        // processing (and to appease the borrow checker).
        let mut uses = BTreeSet::default();
        for inst in body.blocks()[block.clone()].insts() {
            let mut vs = <F as Func>::values(&*body);
            let mut vals = vs[inst].values(body);
            while let Some(w) = vals.next() {
                uses.insert((**w).clone());
            }
        }
        let mut ks = body.blocks();
        let mut vals = ks[block.clone()].term().values(body) ;
        while let Some(u) = vals.next(){
            uses.insert((&**u).clone());
        }
        drop(vals);
        drop(ks);

        for u in uses {
            self.visit_use(body, block.clone(), u);
        }
    }

    fn visit_use(&mut self, body: &mut F, block: F::Block, value: F::Value) {
        if self.value_map.contains_key(&(block.clone(), value.clone())) {
            return;
        }
        // if body.blocks[block].insts.binary_search_by(|a|a.index().cmp(&value.value.index())).is_ok() {
        //     eprintln!("in block value: {}@{}",value.value.index(),block.index());
        //     return;
        // }
        for i in body.blocks()[block.clone()].insts() {
            if i == value {
                return;
            }
        }
        // eprintln!("{:?}",body.blocks[block].insts.iter().map(|a|a.index()).collect::<Vec<_>>());
        self.new_args
            .entry(block.clone())
            .or_default()
            .push(value.clone());

        // Create a placeholder value.
        let ty = <F as Func>::values(&*body)[value.clone()].ty(body);
        let blockparam = body.add_blockparam(block.clone(), ty);
        self.value_map
            .insert((block.clone(), value.clone()), blockparam);

        // Recursively visit preds and use the value there, to ensure
        // they have the value available as well.
        for pred in preds(&*body, block).collect::<Vec<_>>() {
            // Don't borrow for whole loop while iterating (`body` is
            // taken as mut by recursion, but we don't add preds).
            self.visit_use(body, pred, value.clone());
        }
    }

    fn update_branch_args(&mut self, body: &mut F) {
        let ks = body.blocks().iter().collect::<Vec<_>>();
        for block in ks {
            let mut blockdata = &mut body.blocks_mut()[block.clone()];
            // if let Some(term) = blockdata.term.as_mut(){
            for target in blockdata.term_mut().targets_mut() {
                for new_arg in self
                    .new_args
                    .get(&target.block())
                    .clone()
                    .into_iter()
                    .flatten()
                {
                    let actual_value = self
                        .value_map
                        .get(&(block.clone(), new_arg.clone()))
                        .cloned()
                        .unwrap_or(new_arg.clone());
                    target.push_value(actual_value);
                }
            }
            // }
        }
    }

    fn update_uses(&mut self, body: &mut F, block: F::Block) {
        let resolve = |value: F::Value| {
            // let value = body.resolve_alias(value);
            let v = self
                .value_map
                .get(&(block.clone(), value.clone()))
                .cloned()
                .unwrap_or(value);
            v
        };
        let is = body.blocks()[block.clone()].insts().collect::<Vec<_>>();
        for inst in is {
            // let inst = body.blocks()[block].insts[i];
            let mut def = <F as Func>::values(&*body)[inst.clone()].clone();
            let mut vals = def.values_mut(body) ;
            while let Some(mut a) = vals.next(){
                **a = resolve((&**a).clone());
            }
            drop(vals);;
            body.values_mut()[inst] = def;
        }
        let mut term = body.blocks()[block.clone()].term().clone();
        let mut vals = term.values_mut(body);
        while let Some(mut a) = vals.next() {
            **a = resolve((&**a).clone());
        }
        drop(vals);
        *body.blocks_mut()[block.clone()].term_mut() = term;
    }

    fn update(&mut self, body: &mut F) {
        self.update_branch_args(body);
        let ks = body.blocks().iter().collect::<Vec<_>>();
        for block in ks {
            self.update_uses(body, block);
        }
    }
}

fn iter_all_same<Item: PartialEq + Eq + Clone, I: Iterator<Item = Item>>(iter: I) -> Option<Item> {
    let mut item = None;
    for val in iter {
        if item.get_or_insert(val.clone()).clone() != val {
            return None;
        }
    }
    item
}
