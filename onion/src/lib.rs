pub fn union<A: Eq>(a: impl Iterator<Item = A>, b: impl Iterator<Item = A>) -> Union<A> {
    let mut vals = a.collect::<Vec<_>>();
    let pos1s = vals.iter().enumerate().map(|a| a.0).collect();
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
    Union { vals, pos1s, pos2s }
}
pub struct Union<A> {
    pub vals: Vec<A>,
    pub pos1s: Vec<usize>,
    pub pos2s: Vec<usize>,
}
