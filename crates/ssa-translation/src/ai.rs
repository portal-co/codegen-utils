use core::ops::ControlFlow;

use alloc::vec;
use alloc::vec::Vec;
use cfg_traits::{Target as CFGTarget, Term as CFGTerm};
use lending_iterator::prelude::*;
use ssa_traits::{HasValues, Target, Term, TypedFunc, TypedValue};
use valser::{AnyKind, ValSer};

use crate::Translator;

pub struct AI<T: ?Sized> {
    pub handler: T,
}
pub trait Handler<F: TypedFunc<Value: Ord>, G: TypedFunc<Value: Clone>> {
    type Kind: AnyKind<Value<G::Value>: Clone> + Clone;
    fn stamp(
        &mut self,
        fty: F::Ty,
        x: Self::Kind,
    ) -> anyhow::Result<(<Self::Kind as AnyKind>::Value<G::Ty>)>;
    fn unstamp(
        &mut self,
        g: <Self::Kind as AnyKind>::Value<G::Ty>,
    ) -> anyhow::Result<(Self::Kind, F::Ty)>;
    fn emit_val<T: AsMut<AI<Self>>>(
        ctx: &mut T,
        i: &mut Vec<(F::Ty, Self::Kind)>,
        g: &mut G,
        f: &F,
        k: <G as cfg_traits::Func>::Block,
        map: &alloc::collections::BTreeMap<<F>::Value, <Self::Kind as AnyKind>::Value<G::Value>>,
        params: &[<Self::Kind as AnyKind>::Value<G::Value>],
        go: impl FnMut(
            &mut T,
            &mut G,
            &F,
            <F>::Block,
            Vec<(F::Ty, Self::Kind)>,
        ) -> anyhow::Result<<G as cfg_traits::Func>::Block>,
        val: &<<F>::Values as core::ops::Index<<F>::Value>>::Output,
    ) -> anyhow::Result<(
        <Self::Kind as AnyKind>::Value<G::Value>,
        <G as cfg_traits::Func>::Block,
    )>;

    fn emit_term<T: AsMut<AI<Self>>>(
        ctx: &mut T,
        i: &mut Vec<(F::Ty, Self::Kind)>,
        g: &mut G,
        f: &F,
        k: <G as cfg_traits::Func>::Block,
        map: &alloc::collections::BTreeMap<<F>::Value, <Self::Kind as AnyKind>::Value<G::Value>>,
        params: &[<Self::Kind as AnyKind>::Value<G::Value>],
        go: impl FnMut(
            &mut T,
            &mut G,
            &F,
            <F>::Block,
            Vec<(F::Ty, Self::Kind)>,
        ) -> anyhow::Result<<G as cfg_traits::Func>::Block>,
        val: &<<<F>::Blocks as core::ops::Index<<F>::Block>>::Output as cfg_traits::Block<F>>::Terminator,
    ) -> anyhow::Result<()>;

    fn emit_target<T: AsMut<AI<Self>>, Gt: Target<G>>(
        ctx: &mut T,
        i: &mut Vec<(F::Ty, Self::Kind)>,
        g: &mut G,
        f: &F,
        k: <G as cfg_traits::Func>::Block,
        map: &alloc::collections::BTreeMap<<F>::Value, <Self::Kind as AnyKind>::Value<G::Value>>,
        params: &[<Self::Kind as AnyKind>::Value<G::Value>],
        mut go: impl FnMut(
            &mut T,
            &mut G,
            &F,
            <F>::Block,
            Vec<(F::Ty, Self::Kind)>,
        ) -> anyhow::Result<<G as cfg_traits::Func>::Block>,
        val: &impl Target<F>,
    ) -> anyhow::Result<Gt> {
        let v: Vec<<Self::Kind as AnyKind>::Value<G::Value>> = HasValues::values(val, f)
            .filter_map::<HKT!(<'b> => <Self::Kind as AnyKind>::Value<G::Value>), _>(|[], v| {
                map.get(&**v).cloned()
            })
            .into_iter()
            .collect::<Vec<_>>();

        let mut is = vec![];
        let mut ps = vec![];
        for w in v {
            let (a, k) = w.to_values();
            let a = a.collect::<Vec<_>>();
            let ControlFlow::Continue(w) =
                <<Self::Kind as AnyKind>::Value<G::Ty> as ValSer<G::Ty>>::from_kind::<()>(
                    k.clone(),
                    &mut a
                        .iter()
                        .cloned()
                        .map(|x| <G as ssa_traits::Func>::values(g)[x].ty(g))
                        .map(ControlFlow::Continue),
                )
            else {
                anyhow::bail!("not enough values")
            };
            let (k, y) = ctx.as_mut().handler.unstamp(w)?;
            ps.push((y, k));
            is.push(a);
        }
        let k = go(ctx, g, f, val.block(), ps)?;

        Ok(Gt::from_values_and_block(is.into_iter().flatten(), k))
    }
}
impl<
        F: TypedFunc<Ty: Clone, Value: Ord>,
        G: TypedFunc<Block: Clone, Value: Clone>,
        H: Handler<F, G> + ?Sized,
    > Translator<F, G> for AI<H>
{
    type Meta = <H::Kind as AnyKind>::Value<G::Value>;

    type Instance = Vec<(F::Ty, H::Kind)>;

    fn add_blockparam(
        &mut self,
        i: &mut Self::Instance,
        g: &mut G,
        f: &F,
        k: <G as cfg_traits::Func>::Block,
        p: <F as TypedFunc>::Ty,
        i2: usize,
    ) -> anyhow::Result<(Self::Meta, <G as cfg_traits::Func>::Block)> {
        i[i2].0 = p.clone();
        let v = self.handler.stamp(p, i[i2].1.clone())?;
        let (a, b) = v.to_values();
        let m = <<H::Kind as AnyKind>::Value<G::Value> as ValSer<G::Value>>::from_kind::<()>(
            b,
            &mut a.map(|a| ControlFlow::Continue(g.add_blockparam(k.clone(), a))),
        );
        let ControlFlow::Continue(m) = m else {
            anyhow::bail!("not emough values")
        };
        Ok((m, k))
    }

    fn emit_val<T: AsMut<Self>>(
        ctx: &mut T,
        i: &mut Self::Instance,
        g: &mut G,
        f: &F,
        k: <G as cfg_traits::Func>::Block,
        map: &alloc::collections::BTreeMap<<F>::Value, Self::Meta>,
        params: &[Self::Meta],
        go: impl FnMut(
            &mut T,
            &mut G,
            &F,
            <F>::Block,
            Self::Instance,
        ) -> anyhow::Result<<G as cfg_traits::Func>::Block>,
        val: &<<F>::Values as core::ops::Index<<F>::Value>>::Output,
    ) -> anyhow::Result<(Self::Meta, <G as cfg_traits::Func>::Block)> {
        H::emit_val(ctx, i, g, f, k, map, params, go, val)
    }

    fn emit_term<T: AsMut<Self>>(
        ctx: &mut T,
        i: &mut Self::Instance,
        g: &mut G,
        f: &F,
        k: <G as cfg_traits::Func>::Block,
        map: &alloc::collections::BTreeMap<<F>::Value, Self::Meta>,
        params: &[Self::Meta],
        go: impl FnMut(
            &mut T,
            &mut G,
            &F,
            <F>::Block,
            Self::Instance,
        ) -> anyhow::Result<<G as cfg_traits::Func>::Block>,
        val: &<<<F>::Blocks as core::ops::Index<<F>::Block>>::Output as cfg_traits::Block<F>>::Terminator,
    ) -> anyhow::Result<()> {
        H::emit_term(ctx, i, g, f, k, map, params, go, val)
    }
}
