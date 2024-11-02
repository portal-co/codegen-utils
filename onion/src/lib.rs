#![no_std]
extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

pub fn union<A: Eq>(mut b: impl Iterator<Item: Iterator<Item = A>>) -> Union<A> {
    let Some(a) = b.next() else {
        return Union {
            vals: vec![],
            poss: vec![],
        };
    };
    let mut vals = a.collect::<Vec<_>>();
    let pos1s = vals.iter().enumerate().map(|a| a.0).collect();
    let mut poss = vec![pos1s];
    for b in b {
        let mut pos2s: Vec<usize> = vec![];
        for b in b {
            match vals
                .iter()
                .enumerate()
                .filter(|(i, a)| !pos2s.contains(&i))
                .find(|(i, x)| **x == b)
            {
                Some(c) => pos2s.push(c.0),
                None => {
                    pos2s.push(vals.len());
                    vals.push(b)
                }
            }
        }
        poss.push(pos2s);
    }
    Union { vals, poss }
}
pub struct Union<A> {
    pub vals: Vec<A>,
    pub poss: Vec<Vec<usize>>,
}
impl<A> Union<A> {
    pub fn create<B, E>(
        &self,
        i: usize,
        default: impl FnMut(&A) -> Result<B, E>,
        vals: impl Iterator<Item = Result<B, E>>,
    ) -> Result<Vec<B>, E> {
        let mut bs: Vec<B> = self.vals.iter().map(default).collect::<Result<_, E>>()?;
        for (j, b) in self.poss[i].iter().cloned().zip(vals) {
            bs[j] = b?;
        }
        return Ok(bs);
    }
}
