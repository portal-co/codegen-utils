pub trait Ast: Sized{
    type Value<Sub>;
    type Control<Sub>;
    fn to_impl(self) -> AstImpl<Self>;
    fn from_impl(a: AstImpl<Self>) ->Self;
}
pub enum AstImpl<A: Ast>{
    Op(A::Value<A>),
    Control(A::Control<A>)
}