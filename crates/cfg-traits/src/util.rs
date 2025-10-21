use crate::*;
use core::cmp::Ordering;
use core::hash::Hash;
#[macro_export]
macro_rules! func_via_cfg {
    (<$($param:ident $([: $($path:path),*])?),*>$i:ident => $t:ty) => {
        pub struct $i<$($param : $($($path)+*)?),*>(pub $crate::FuncViaCfg<$t,Self>);
        const _: () = {
            impl<$($param : $($($path)+*)?),*> $crate::Deref for $i<$($param),*>{
                type Target = $crate::FuncViaCfg<$t,Self>;
                fn deref(&self) -> &$crate::FuncViaCfg<$t,Self>{
                    match self{
                        $i(a) => a,
                    }
                }
            }
        }
    };
}
pub struct FuncViaCfg<T, W: Deref<Target = Self> + Func + ?Sized> {
    pub cfg: T,
    pub entry_block: W::Block,
}
pub trait CfgOf: Func + Deref<Target = FuncViaCfg<Self::Cfg, Self>> {
    type Cfg;
}
impl<T, W: Deref<Target = FuncViaCfg<T, W>> + Func> CfgOf for W {
    type Cfg = T;
}
impl<T: Clone, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: Clone> + ?Sized> Clone
    for FuncViaCfg<T, W>
{
    fn clone(&self) -> Self {
        Self {
            cfg: self.cfg.clone(),
            entry_block: self.entry_block.clone(),
        }
    }
}
impl<T: PartialEq, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: PartialEq> + ?Sized> PartialEq
    for FuncViaCfg<T, W>
{
    fn eq(&self, other: &Self) -> bool {
        self.cfg == other.cfg && self.entry_block == other.entry_block
    }
}
impl<T: Eq, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: Eq> + ?Sized> Eq
    for FuncViaCfg<T, W>
{
}
impl<T: PartialOrd, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: PartialOrd> + ?Sized>
    PartialOrd for FuncViaCfg<T, W>
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        match self.cfg.partial_cmp(&other.cfg) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.entry_block.partial_cmp(&other.entry_block)
    }
}
impl<T: Ord, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: Ord> + ?Sized> Ord
    for FuncViaCfg<T, W>
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self.cfg.cmp(&other.cfg) {
            Ordering::Equal => return self.entry_block.cmp(&other.entry_block),
            a => return a,
        }
    }
}
impl<T: Hash, W: Deref<Target = FuncViaCfg<T, W>> + Func<Block: Hash> + ?Sized> Hash
    for FuncViaCfg<T, W>
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.cfg.hash(state);
        self.entry_block.hash(state);
    }
}
