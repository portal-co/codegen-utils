#![no_std]

use either::Either;

pub trait Sift<T> {
    type Residue;
    fn sift(self) -> Result<T, Self::Residue>;
    fn of(t: T) -> Self;
    fn lift(r: Self::Residue) -> Self;
}
impl<T, U, A: Sift<T, Residue: Sift<U>>> Sift<Either<T, U>> for A {
    type Residue = <<A as Sift<T>>::Residue as Sift<U>>::Residue;

    fn sift(self) -> Result<Either<T, U>, Self::Residue> {
        match self.sift() {
            Ok(a) => Ok(Either::Left(a)),
            Err(b) => b.sift().map(Either::Right),
        }
    }

    fn of(t: Either<T, U>) -> Self {
        match t {
            Either::Left(a) => <A as Sift<T>>::of(a),
            Either::Right(b) => <A as Sift<T>>::lift(<<A as Sift<T>>::Residue as Sift<U>>::of(b)),
        }
    }

    fn lift(r: Self::Residue) -> Self {
        <A as Sift<T>>::lift(<<A as Sift<T>>::Residue as Sift<U>>::lift(r))
    }
}
