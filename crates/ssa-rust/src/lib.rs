use arena_traits::{Arena, IndexIter};
use cfg_traits::{Block as CFGBlock, Target as CFGTarget, Term as CFGTerm};
use either::Either;
use lending_iterator::prelude::*;
use proc_macro2::{Span, TokenStream};
use quasiquote::quasiquote;
use quote::{format_ident, quote};
use relooper::{BranchMode, RelooperLabel, ShapedBlock};
use ssa_traits::{Block, Target, Term, TypedBlock, TypedFunc, TypedValue};
use std::iter::empty;
use syn::{Ident, Lifetime};
fn term(b: &BranchMode) -> TokenStream {
    match b {
        relooper::BranchMode::LoopBreak(l) => {
            let l = Lifetime::new(&format!("'l{}", l), Span::call_site());
            quote! {
                break #l;
            }
        }
        relooper::BranchMode::LoopBreakIntoMulti(l) => {
            let l = Lifetime::new(&format!("'l{}", l), Span::call_site());
            quote! {
                break #l;
            }
        }
        relooper::BranchMode::LoopContinue(l) => {
            let l = Lifetime::new(&format!("'l{}", l), Span::call_site());
            quote! {
                continue #l;
            }
        }
        relooper::BranchMode::LoopContinueIntoMulti(l) => {
            let l = Lifetime::new(&format!("'l{}", l), Span::call_site());
            quote! {
                continue #l;
            }
        }
        relooper::BranchMode::MergedBranch => {
            quote! {}
        }
        relooper::BranchMode::MergedBranchIntoMulti => quote! {},
        relooper::BranchMode::SetLabelAndBreak => quote! {
            break 'cff;
        },
    }
}
pub trait RsFunc:
    TypedFunc<
        Ty: Rs<Self>,
        Block: RsId<Self> + Ord + Clone + RelooperLabel,
        Value: RsId<Self> + Clone,
        Values: Arena<Self::Value, Output: Rs<Self>>,
        Blocks: Arena<Self::Block, Output: Block<Self, Terminator: RsTerm<Self>>>,
    > + Sized
{
}
impl<
        T: TypedFunc<
                Ty: Rs<Self>,
                Block: RsId<Self> + Ord + Clone + RelooperLabel,
                Value: RsId<Self> + Clone,
                Values: Arena<Self::Value, Output: Rs<Self>>,
                Blocks: Arena<Self::Block, Output: Block<Self, Terminator: RsTerm<Self>>>,
            > + Sized,
    > RsFunc for T
{
}
pub trait Rs<F: RsFunc> {
    fn rs(&self, f: &F) -> anyhow::Result<TokenStream>;
}
pub trait RsId<F: RsFunc> {
    fn rs(&self, f: &F) -> anyhow::Result<Ident>;
}
pub trait RsTerm<F: RsFunc> {
    fn rs_term(
        &self,
        f: &F,
        go: impl FnMut(F::Block) -> anyhow::Result<TokenStream>,
    ) -> anyhow::Result<TokenStream>;
}
pub fn render_target<R: RsFunc>(
    t: &impl Target<R>,
    f: &R,
    go: &mut impl FnMut(R::Block) -> anyhow::Result<TokenStream>,
    prepend: impl Iterator<Item: Rs<R>>,
) -> anyhow::Result<TokenStream> {
    let vars = prepend
        .map(|x| x.rs(f))
        .chain(
            t.values(f)
                .map::<HKT!(<'b> => anyhow::Result<TokenStream>), _>(|[], a| {
                    let a = format_ident!("V{}", a.rs(f)?);
                    anyhow::Ok(quasiquote!(#a .take()))
                })
                .into_iter(),
        )
        .enumerate()
        .map(|(i, a)| {
            let i = format_ident!("P{}_{i}", t.block().rs(f)?);
            Ok(quasiquote! {
                #i = #{a?};
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok(quasiquote! {
        #(#vars);*
        #{go(t.block())?}
    })
}
pub fn go<F: RsFunc>(params: &[TokenStream], f: &F, e: F::Block) -> anyhow::Result<TokenStream> {
    Ok(quasiquote! {
        #{
            let k = f.blocks().iter().flat_map(|a|{
                if a == e{
                    Either::Left(params.iter().enumerate().map(move|(i,p)|Ok(quasiquote!(let mut #{format_ident!("P{}_{i}",a.rs(f)?)} = Some(#p)))))
                }else{
                    Either::Right(f.blocks()[a].params().map(|(a,_)|a).enumerate().map(move|(i,t)|Ok(quasiquote!(let mut #{format_ident!("P{}_{i}",a.rs(f)?)}: Option<#{t.rs(f)?}> = None))).collect::<Vec<_>>().into_iter())
                }
            }).chain(f.values().iter().map(|v|{
                let ty = f.values()[v.clone()].ty(f).rs(f)?;
                Ok(quote!{
                    let mut #{format_ident!("V{}",v.rs(f)?)}: Option<#ty> = None;
                })
            })).collect::<anyhow::Result<Vec<_>>>()?;
            quote! {
                #(#k);*
            }
        };
        let mut cff = 0usize;
        #{block(f,ssa_reloop::go(f,e).as_ref())?}
    })
}
pub fn idx_of<F: RsFunc>(f: &F, k: F::Block) -> usize {
    f.blocks()
        .iter()
        .enumerate()
        .find_map(|(i, l)| if l == k { Some(i) } else { None })
        .unwrap()
}
pub fn block<F: RsFunc>(f: &F, k: &ShapedBlock<F::Block>) -> anyhow::Result<TokenStream> {
    match k {
        ShapedBlock::Loop(l) => {
            let r = block(f, &l.inner.as_ref())?;
            let next = l.next.as_ref();
            let next = match next {
                None => Default::default(),
                Some(a) => block(f, a)?,
            };
            let l = Lifetime::new(&format!("'l{}", l.loop_id), Span::call_site());
            Ok(quote! {
                #l : loop{
                    #r
                };
                #next;
            })
        }
        ShapedBlock::Multiple(k) => {
            let initial = k.handled.iter().enumerate().flat_map(|(a, b)| {
                b.labels.iter().map(move |l| {
                    let l = idx_of(f, *l);
                    quote! {
                        #l => #a
                    }
                })
            });
            let cases = k
                .handled
                .iter()
                .enumerate()
                .map(|(a, i)| {
                    let ib = block(f, &i.inner)?;
                    let ic = if i.break_after {
                        quote! {}
                    } else {
                        quote! {
                            cff2 += 1;
                            continue 'cff
                        }
                    };
                    Ok(quote! {
                        #a => {
                            #ib;
                            #ic;
                        }
                    })
                })
                .collect::<anyhow::Result<Vec<_>>>()?;
            Ok(quote! {
                let mut cff2 = match cff{
                    #(#initial),*,
                    _ => unreachable!()
                };
                'cff: loop{
                    match cff2{
                        #(#cases),*,
                        _ => unreachable!()
                    };
                    break 'cff;
                };
            })
        }
        ShapedBlock::Simple(s) => {
            let immediate = match s.immediate.as_ref() {
                None => Default::default(),
                Some(a) => block(f, a)?,
            };
            let next = match s.next.as_ref() {
                None => Default::default(),
                Some(a) => block(f, a)?,
            };
            let stmts = f.blocks()[s.label]
                .insts()
                .map(|v| {
                    Ok(quasiquote! {
                        #{format_ident!("V{}",v.rs(f)?)} = Some(#{f.values()[v].rs(f)?})
                    })
                })
                .collect::<anyhow::Result<Vec<_>>>()?;
            let term = f.blocks()[s.label].term().rs_term(f, |k| {
                let br = term(
                    &s.branches
                        .get(&k)
                        .cloned()
                        .unwrap_or(relooper::BranchMode::MergedBranch),
                );
                let bi = idx_of(f, k);
                Ok(quote! {
                    cff = #bi;
                    #br
                })
            })?;
            Ok(quote! {
                #(#stmts);*;
                #term;
                #immediate;
                #next;
            })
        }
    }
}
#[cfg(feature = "id-arena")]
impl<F: RsFunc, T> RsId<F> for id_arena::Id<T> {
    fn rs(&self, f: &F) -> anyhow::Result<Ident> {
        Ok(format_ident!("V{}", self.index()))
    }
}
pub trait RsOp<F: RsFunc> {
    fn rs_op(
        &self,
        f: &F,
        all: &[impl RsId<F>],
        blargs: &[F::Block],
    ) -> anyhow::Result<TokenStream>;
}
#[cfg(feature = "ssa-canon")]
impl<F: RsFunc<Block = id_arena::Id<ssa_canon::Block<O, T, Y>>>, O: RsOp<F>, T, Y> Rs<F>
    for ssa_canon::Value<O, T, Y>
{
    fn rs(&self, f: &F) -> anyhow::Result<TokenStream> {
        match self {
            ssa_canon::Value::Op(o, args, q, _) => o.rs_op(f, &args, q.as_slice()),
            ssa_canon::Value::Param(i, a, _) => {
                Ok(quasiquote!(#{format_ident!("P{}_{i}",a.rs(f)?)}.take().unwrap()))
            }
        }
    }
}
#[cfg(feature = "ssa-canon")]
impl<
        O: RsOp<ssa_canon::Func<O, T, Y>>,
        T: Term<ssa_canon::Func<O, T, Y>, Target = ssa_canon::Target<O, T, Y>>,
        Y: Clone,
    > RsTerm<ssa_canon::Func<O, T, Y>> for ssa_canon::Target<O, T, Y>
where
    ssa_canon::Func<O, T, Y>: RsFunc<Block = id_arena::Id<ssa_canon::Block<O, T, Y>>>,
{
    fn rs_term(
        &self,
        f: &ssa_canon::Func<O, T, Y>,
        mut go: impl FnMut(
            <ssa_canon::Func<O, T, Y> as cfg_traits::Func>::Block,
        ) -> anyhow::Result<TokenStream>,
    ) -> anyhow::Result<TokenStream> {
        render_target(self, f, &mut go, empty::<ssa_canon::Value<O, T, Y>>())
    }
}
impl<F: RsFunc, A: RsOp<F>, B: RsOp<F>> RsOp<F> for Either<A, B> {
    fn rs_op(
        &self,
        f: &F,
        all: &[impl RsId<F>],
        blargs: &[F::Block],
    ) -> anyhow::Result<TokenStream> {
        match self {
            Either::Left(a) => a.rs_op(f, all, blargs),
            Either::Right(b) => b.rs_op(f, all, blargs),
        }
    }
}
