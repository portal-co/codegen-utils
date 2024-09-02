use arena_traits::Arena;

// use ssa_traits::TypedFunc;
use ssa_traits::*;

pub fn cc<F: CCFunc>(s: &F) -> anyhow::Result<String> {
    let params = s.blocks()[s.entry()]
        .params()
        .enumerate()
        .map(|(a, (b,_))| Ok(format!("{} {}", b.c(s)?, kp(&s.entry(), a, s)?)))
        .collect::<anyhow::Result<Vec<_>>>()?
        .join(",");
    let vars = s
        .blocks()
        .iter()
        .filter(|b| *b != s.entry())
        .flat_map(|c| {
            s.blocks()[c.clone()]
                .params()
                .enumerate()
                .map(move |(a, (b,_))| Ok(format!("{} {}", b.c(s)?, kp(&c, a, s)?)))
        })
        .chain(
            s.values()
                .iter()
                .map(|v| Ok(format!("{} {}", s.values()[v.clone()].ty(s).c(s)?, v.c(s)?))),
        )
        .collect::<anyhow::Result<Vec<_>>>()?
        .join(",");
    let body = s
        .blocks()
        .iter()
        .map(|b| {
            let vals = s.blocks()[b.clone()]
                .insts()
                .map(|i| Ok(format!("{} = {}", i.c(s)?, s.values()[i].c(s)?)))
                .collect::<anyhow::Result<Vec<_>>>()?
                .join(";");
            let term = s.blocks()[b.clone()].term().c(s)?;
            Ok(format!("BB{}: {vals};{term}", b.c(s)?))
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .join(";");
    Ok(format!(
        r"
    ({params}){{
    {vars};
    goto BB{};
        {body}
    }}
    ",
        s.entry().c(s)?
    ))
}
pub trait C<F: ?Sized> {
    fn c(&self, f: &F) -> anyhow::Result<String>;
}
pub fn kp<F: ?Sized>(a: &impl C<F>, b: usize, f: &F) -> anyhow::Result<String> {
    Ok(format!("_{}_{b}", a.c(f)?))
}
pub trait CCFunc:
    TypedFunc<
    Ty: C<Self>,
    Block: C<Self> + Ord + Clone,
    Value: C<Self> + Clone,
    Values: Arena<Self::Value, Output: C<Self>>,
    Blocks: Arena<Self::Block, Output: Block<Self, Terminator: C<Self>>>,
>
{
}
impl<
        T: TypedFunc<
            Ty: C<Self>,
            Block: C<Self> + Ord + Clone,
            Value: C<Self> + Clone,
            Values: Arena<Self::Value, Output: C<Self>>,
            Blocks: Arena<Self::Block, Output: Block<Self, Terminator: C<Self>>>,
        >,
    > CCFunc for T
{
}
pub fn render_target<C: CCFunc>(t: &impl Target<C>, c: &C) -> anyhow::Result<String> {
    let args = t
        .values(c)
        .enumerate()
        .map(|(i, v)| {
            let k = kp(&t.block(), i, c)?;

            Ok(format!("{} = {}", k, v.c(c)?))
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .join(";");
    Ok(format!("{args};goto BB{};", t.block().c(c)?))
}
pub trait COp<F: ?Sized> {
    fn c(&self, args: &[impl C<F>], f: &F) -> anyhow::Result<String>;
}
#[cfg(feature = "id-arena")]
impl<F: ?Sized, T> C<F> for id_arena::Id<T> {
    fn c(&self, f: &F) -> anyhow::Result<String> {
        Ok(format!("x{}", self.index()))
    }
}
#[cfg(feature = "ssa-canon")]
impl<F: ?Sized, O: COp<F>, T, Y> C<F> for ssa_canon::Value<O, T, Y> {
    fn c(&self, f: &F) -> anyhow::Result<String> {
        use ssa_canon::Value;
        match self {
            Value::Op(o, a, _) => o.c(&a, f),
            Value::Param(n, k, _) => kp(k, *n, f),
        }
    }
}
#[cfg(feature = "ssa-canon")]
impl<O, T: Term<ssa_canon::Func<O, T, Y>, Target = ssa_canon::Target<O, T, Y>>, Y: Clone>
    C<ssa_canon::Func<O, T, Y>> for ssa_canon::Target<O, T, Y>
where
    ssa_canon::Func<O, T, Y>: CCFunc,
{
    fn c(&self, f: &ssa_canon::Func<O, T, Y>) -> anyhow::Result<String> {
        render_target(self, f)
    }
}
