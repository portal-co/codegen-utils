use crate::{Func, HasValues};

pub trait OpValue<F: Func,O>{
    type Residue;
    type Capture: HasValues<F>;
    type Spit;
    fn disasm(self, f: &mut F) -> Result<(O,Self::Capture,Self::Spit),Self::Residue>;
    fn of(f: &mut F,o: O, c: Self::Capture,s: Self::Spit) -> Self;
    fn lift(f: &mut F,r: Self::Residue) -> Self;
}

