use std::collections::HashMap;
use pliron::{
    basic_block::BasicBlock,
    context::{Context, Ptr}, op::Op, value::Value,
};
use ssa_traits::Func;
pub trait PlironCompatOp<F: Func>: Op {
    fn to_ssa_traits(
        &self,
        ctx: &Context,
        f: &mut F,
        k: F::Block,
        values: &HashMap<Value, F::Value>,
        blocks: &HashMap<Ptr<BasicBlock>, F::Block>,
    ) -> anyhow::Result<(F::Value, F::Block)>;
    fn verify(_op: &dyn Op, _ctx: &Context) -> pliron::result::Result<()>
    where
        Self: Sized,
    {
        Ok(())
    }
}
