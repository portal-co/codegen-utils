use std::{
    collections::BTreeMap,
    ops::{Index, IndexMut},
};

use arena_traits::Arena;
use ssa_traits::TypedBlock;
use ssa_traits::{Block, Func, TypedFunc};

pub trait Translator<F: TypedFunc, G: Func> {
    type Meta;
    type Instance;
    fn add_blockparam(
        &mut self,
        i: &Self::Instance,
        g: &mut G,
        f: &F,
        k: G::Block,
        p: F::Ty,
    ) -> anyhow::Result<(Self::Meta, G::Block)>;
    fn emit_val<T: AsMut<Self>>(
        ctx: &mut T,
        i: &Self::Instance,
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
        i: &Self::Instance,
        g: &mut G,
        f: &F,
        k: G::Block,
        map: &BTreeMap<F::Value, Self::Meta>,
        params: &[Self::Meta],
        go: impl FnMut(&mut T, &mut G, &F, F::Block, Self::Instance) -> anyhow::Result<G::Block>,
        val: &<<F::Blocks as Index<F::Block>>::Output as Block<F>>::Terminator,
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
        i: T::Instance,
    ) -> anyhow::Result<G::Block> {
        loop {
            if let Some(v) = self.in_map.get(&(b.clone(), i.clone())) {
                return Ok(v.clone());
            }
            let mut v = g.blocks_mut().alloc(Default::default());
            self.in_map.insert((b.clone(), i.clone()), v.clone());
            let mut vals = BTreeMap::new();
            let mut params = vec![];
            for (fp,_) in f.blocks()[b.clone()].params() {
                let val;
                (val, v) = self.wrapped.add_blockparam(&i, g, f, v.clone(), fp)?;
                params.push(val);
            }
            for val2 in f.blocks()[b.clone()].insts() {
                let w = &f.values()[val2.clone()];
                let val;
                (val, v) = T::emit_val(self, &i, g, f, v.clone(), &vals, &params, Self::go, w)?;
                vals.insert(val2, val);
            }
            let t = f.blocks()[b.clone()].term();
            T::emit_term(self, &i, g, f, v.clone(), &vals, &params, Self::go, &t)?;
        }
    }
}
