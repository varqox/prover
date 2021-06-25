#[derive(Clone, Copy)]
struct TupleIteratorState {
    idx_sum: usize,
    next_idx: usize,
}

pub(crate) struct TupleIterator<I: Iterator> {
    iter: I,
    arity: usize,
    elems: Vec<I::Item>,
    // recursive generator state
    elems_len: Option<usize>,
    next_idx_sum: usize,
    state: Vec<TupleIteratorState>,
    prefix: Vec<I::Item>,
}

impl<I, T> TupleIterator<I>
where
    T: Clone,
    I: Iterator<Item = T>,
{
    pub(crate) fn new(iter: I, arity: usize) -> Self {
        Self {
            iter,
            arity,
            elems: Vec::new(),
            elems_len: None,
            next_idx_sum: 0,
            state: Vec::new(),
            prefix: Vec::new(),
        }
    }

    fn get(&mut self, idx: usize) -> Option<I::Item> {
        while self.elems.len() <= idx {
            match self.iter.next() {
                Some(elem) => {
                    self.elems.push(elem);
                }
                None => {
                    self.elems_len = Some(self.elems.len());
                    return None;
                }
            }
        }
        Some(self.elems[idx].clone())
    }
}

impl<I, T> Iterator for TupleIterator<I>
where
    T: Clone,
    I: Iterator<Item = T>,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.state.is_empty() {
                if self.arity == 0 && self.next_idx_sum > 0 {
                    return None;
                }
                if let Some(elems_len) = self.elems_len {
                    if self.arity + self.next_idx_sum > self.arity * elems_len {
                        return None; // Already iterated all tuples
                    }
                }
                self.state.push(TupleIteratorState {
                    idx_sum: self.next_idx_sum,
                    next_idx: 0,
                });
                self.next_idx_sum += 1;
            }
            let TupleIteratorState { idx_sum, next_idx } = self.state.last().copied().unwrap();
            if next_idx == 0 {
                if let Some(elems_len) = self.elems_len {
                    if idx_sum > (elems_len - 1) * (self.arity - self.prefix.len()) {
                        // There is no chance of satisfying idx_sum
                        self.state.pop();
                        continue;
                    }
                }
                if self.prefix.len() == self.arity {
                    let res = self.prefix.clone();
                    self.state.pop();
                    return Some(res);
                }
                let idx = if self.prefix.len() + 1 == self.arity {
                    idx_sum
                } else {
                    0
                };
                match self.get(idx) {
                    Some(elem) => {
                        self.prefix.push(elem);
                        self.state.last_mut().unwrap().next_idx = idx + 1;
                        self.state.push(TupleIteratorState {
                            idx_sum: idx_sum - idx,
                            next_idx: 0,
                        });
                        continue;
                    }
                    None => {
                        self.state.pop();
                        continue;
                    }
                }
            }
            // Returned from a recursive call
            self.prefix.pop();
            // Check bounds
            if next_idx > idx_sum {
                self.state.pop();
                continue;
            }
            if let Some(elems_len) = self.elems_len {
                if next_idx > elems_len {
                    self.state.pop();
                    continue;
                }
            }
            // Proceed with next iteration
            self.state.last_mut().unwrap().next_idx += 1;
            match self.get(next_idx) {
                Some(elem) => {
                    self.prefix.push(elem);
                    self.state.push(TupleIteratorState {
                        idx_sum: idx_sum - next_idx,
                        next_idx: 0,
                    });
                    continue;
                }
                None => {
                    self.state.pop();
                    continue;
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::tuple_iterator::TupleIterator;

    #[test]
    fn tuple_iterator_general() {
        for end in 0..7_usize {
            for arity in 0..7_u32 {
                let v = TupleIterator::new(0..end, arity as usize).collect::<Vec<_>>();
                assert_eq!(v.len(), end.pow(arity));
                // Correct elements
                let mut prev_sum = 0;
                for tuple in v.iter() {
                    let sum = tuple.iter().fold(0, |acc, x| acc + x);
                    assert!(prev_sum <= sum);
                    prev_sum = sum;
                    let sum = sum as i64;
                    let end = end as i64;
                    let arity = arity as i64;
                    assert!(sum <= arity * (end - 1));
                }
                // No repetitions
                let s = v.into_iter().collect::<HashSet<_>>();
                assert_eq!(s.len(), end.pow(arity));
            }
        }
    }

    #[test]
    fn tuple_iterator_infinite() {
        assert_eq!(TupleIterator::new(0.., 0).collect::<Vec<_>>(), vec![vec![]]);
        assert_eq!(
            TupleIterator::new(0.., 2).take(15).collect::<Vec<_>>(),
            vec![
                vec![0, 0],
                vec![0, 1],
                vec![1, 0],
                vec![0, 2],
                vec![1, 1],
                vec![2, 0],
                vec![0, 3],
                vec![1, 2],
                vec![2, 1],
                vec![3, 0],
                vec![0, 4],
                vec![1, 3],
                vec![2, 2],
                vec![3, 1],
                vec![4, 0]
            ]
        );
        assert_eq!(
            TupleIterator::new(0.., 3).take(16).collect::<Vec<_>>(),
            vec![
                vec![0, 0, 0],
                vec![0, 0, 1],
                vec![0, 1, 0],
                vec![1, 0, 0],
                vec![0, 0, 2],
                vec![0, 1, 1],
                vec![0, 2, 0],
                vec![1, 0, 1],
                vec![1, 1, 0],
                vec![2, 0, 0],
                vec![0, 0, 3],
                vec![0, 1, 2],
                vec![0, 2, 1],
                vec![0, 3, 0],
                vec![1, 0, 2],
                vec![1, 1, 1]
            ]
        );
        for arity in 1..10_usize {
            let v = TupleIterator::new(0.., arity)
                .take(1000)
                .collect::<Vec<_>>();
            // Correct elements
            let mut prev_sum = 0;
            for tuple in v.iter() {
                let sum = tuple.iter().sum();
                assert!(prev_sum <= sum);
                prev_sum = sum;
            }
            // No repetitions
            let s = v.into_iter().collect::<HashSet<_>>();
            assert_eq!(s.len(), 1000);
        }
    }
}
