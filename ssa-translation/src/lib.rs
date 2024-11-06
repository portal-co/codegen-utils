#![no_std]
extern crate alloc;

use core::{
    ops::{Index, IndexMut},
};
use  alloc::{
    collections::BTreeMap, vec,
};


use arena_traits::{Arena, IndexAlloc};
use ssa_traits::TypedBlock;
use ssa_traits::{Block, Func, TypedFunc};
use cfg_traits::{Block as CFGBlock};
use valser::{AnyKind, ValSer};

pub mod ai;

pub trait EqIter: IntoIterator + FromIterator<Self::Item> {}
impl<T: IntoIterator + FromIterator<Self::Item>> EqIter for T {}

pub trait CarryTranslator<F: TypedFunc, G: Func>:
    Translator<F, G, Meta: ValSer<G::Value>, Instance: EqIter<Item = (F::Ty, <Self::Meta as ValSer<G::Value>>::Kind)>>
{
}
impl<
        F: TypedFunc,
        G: Func,
        X: Translator<F, G, Meta: ValSer<G::Value>, Instance: EqIter<Item = (F::Ty, <Self::Meta as ValSer<G::Value>>::Kind)>>,
    > CarryTranslator<F, G> for X
{
}
pub trait Translator<F: TypedFunc, G: Func> {
    type Meta;
    type Instance;
    fn add_blockparam(
        &mut self,
        i: &mut Self::Instance,
        g: &mut G,
        f: &F,
        k: G::Block,
        p: F::Ty,
        i2: usize,
    ) -> anyhow::Result<(Self::Meta, G::Block)>;
    fn emit_val<T: AsMut<Self>>(
        ctx: &mut T,
        i: &mut Self::Instance,
        g: &mut G,
        f: &F,
        k: G::Block,
        map: &BTreeMap<F::Value, Self::Meta>,
        params: &[Self::Meta],
        go: impl FnMut(&mut T, &mut G, &F, F::Block, Self::Instance) -> anyhow::Result<G::Block>,
        val: &<F::Values as Index<F::Value>>::Output,
    ) -> anyhow::Result<(Self::Meta, G::Block)>;
    fn emit_term<T: AsMut<Self>>(
        ctx: &mut T,
        i: &mut Self::Instance,
        g: &mut G,
        f: &F,
        k: G::Block,
        map: &BTreeMap<F::Value, Self::Meta>,
        params: &[Self::Meta],
        go: impl FnMut(&mut T, &mut G, &F, F::Block, Self::Instance) -> anyhow::Result<G::Block>,
        val: &<<F::Blocks as Index<F::Block>>::Output as CFGBlock<F>>::Terminator,
    ) -> anyhow::Result<()>;
}
pub struct State<F: TypedFunc, G: Func, T: Translator<F, G>> {
    pub wrapped: T,
    pub in_map: BTreeMap<(F::Block, T::Instance), G::Block>,
}
impl<
        F: TypedFunc<Block: Ord + Clone, Values: Arena<F::Value, Output: Sized>>,
        G: Func<Block: Ord + Clone, Blocks: Arena<G::Block, Output: Default>>,
        T: Translator<F, G, Instance: Ord + Clone>,
    > AsMut<T> for State<F, G, T>
{
    fn as_mut(&mut self) -> &mut T {
        &mut self.wrapped
    }
}
impl<
        F: TypedFunc<Block: Ord + Clone, Values: Arena<F::Value, Output: Sized>, Value: Clone + Ord>,
        G: Func<Block: Ord + Clone, Blocks: Arena<G::Block, Output: Default>>,
        T: Translator<F, G, Instance: Ord + Clone>,
    > State<F, G, T>
{
    pub fn go(
        &mut self,
        g: &mut G,
        f: &F,
        b: F::Block,
        mut i: T::Instance,
    ) -> anyhow::Result<G::Block> {
        loop {
            if let Some(v) = self.in_map.get(&(b.clone(), i.clone())) {
                return Ok(v.clone());
            }
            let mut i = i.clone();
            let mut v = g.blocks_mut().alloc(Default::default());
            self.in_map.insert((b.clone(), i.clone()), v.clone());
            let mut vals = BTreeMap::new();
            let mut params = vec![];
            for (i2,(fp, _)) in f.blocks()[b.clone()].params().enumerate() {
                let val;
                (val, v) = self.wrapped.add_blockparam(&mut i, g, f, v.clone(), fp,i2)?;
                params.push(val);
            }
            for val2 in f.blocks()[b.clone()].insts() {
                let w = &f.values()[val2.clone()];
                let val;
                (val, v) = T::emit_val(self, &mut i, g, f, v.clone(), &vals, &params, Self::go, w)?;
                vals.insert(val2, val);
            }
            let ks = f.blocks();
            let t = ks[b.clone()].term();
            T::emit_term(self, &mut i, g, f, v.clone(), &vals, &params, Self::go, &t)?;
        }
    }
}
