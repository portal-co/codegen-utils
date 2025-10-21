use crate::{Func, HasValues};
use either::Either;
pub trait OpValue<F: Func, O>: Sized {
    type Residue;
    type Capture: HasValues<F>;
    type Spit;
    fn disasm(self, f: &mut F) -> Result<(O, Self::Capture, Self::Spit), Self::Residue>;
    fn of(f: &mut F, o: O, c: Self::Capture, s: Self::Spit) -> Option<Self>;
    fn lift(f: &mut F, r: Self::Residue) -> Option<Self>;
}
impl<F: Func, A, B, T: OpValue<F, A, Residue: OpValue<F, B>>> OpValue<F, Either<A, B>> for T {
    type Residue = <T::Residue as OpValue<F, B>>::Residue;
    type Capture = Either<T::Capture, <T::Residue as OpValue<F, B>>::Capture>;
    type Spit = Either<T::Spit, <T::Residue as OpValue<F, B>>::Spit>;
    fn disasm(self, f: &mut F) -> Result<(Either<A, B>, Self::Capture, Self::Spit), Self::Residue> {
        match self.disasm(f) {
            Ok((a, b, c)) => Ok((Either::Left(a), Either::Left(b), Either::Left(c))),
            Err(d) => match d.disasm(f) {
                Err(e) => Err(e),
                Ok((a, b, c)) => Ok((Either::Right(a), Either::Right(b), Either::Right(c))),
            },
        }
    }
    fn of(f: &mut F, o: Either<A, B>, c: Self::Capture, s: Self::Spit) -> Option<T> {
        match (o, c, s) {
            (Either::Left(o), Either::Left(c), Either::Left(s)) => {
                <T as OpValue<F, A>>::of(f, o, c, s)
            }
            (Either::Right(o), Either::Right(c), Either::Right(s)) => {
                <T::Residue as OpValue<F, B>>::of(f, o, c, s)
                    .and_then(|b| <T as OpValue<F, A>>::lift(f, b))
            }
            _ => None,
        }
    }
    fn lift(f: &mut F, r: Self::Residue) -> Option<T> {
        <T::Residue as OpValue<F, B>>::lift(f, r).and_then(|b| <T as OpValue<F, A>>::lift(f, b))
    }
}
