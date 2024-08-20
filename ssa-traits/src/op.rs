use crate::{Func, HasValues};

pub trait OpValue<F: Func,O>{
    type Residue;
    type Capture: HasValues<F>;
    fn disasm(self) -> Result<(O,Self::Capture),Self::Residue>;
    fn of(o: O, c: Self::Capture) -> Self;
    fn lift(r: Self::Residue) -> Self;
}

