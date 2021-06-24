pub(crate) struct Interleave<T> {
    iters: Vec<(T, usize)>,
    next_idx: (usize, usize),
}

impl<T: Iterator> Interleave<T> {
    pub(crate) fn new(iters: Vec<(T, usize)>) -> Self {
        Self {
            iters,
            next_idx: (0, 0),
        }
    }
}

impl<T: Iterator> Iterator for Interleave<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.iters.is_empty() {
            let (iter, share) = &mut self.iters[self.next_idx.0];
            match iter.next() {
                Some(elem) => {
                    self.next_idx.1 += 1;
                    if self.next_idx.1 >= *share {
                        self.next_idx.0 += 1;
                        self.next_idx.1 = 0;
                        if self.next_idx.0 == self.iters.len() {
                            self.next_idx = (0, 0);
                        }
                    }
                    return Some(elem);
                }
                None => {
                    self.iters.remove(self.next_idx.0);
                    self.next_idx.1 = 0;
                    if self.next_idx.0 == self.iters.len() {
                        self.next_idx.0 = 0;
                    }
                    continue;
                }
            }
        }
        None
    }
}

#[test]
fn interleave() {
    assert_eq!(
        Interleave::new(vec![(10..13, 1), (20..29, 2), (30..39, 3)]).collect::<Vec<_>>(),
        vec![10, 20, 21, 30, 31, 32, 11, 22, 23, 33, 34, 35, 12, 24, 25, 36, 37, 38, 26, 27, 28]
    );
}
