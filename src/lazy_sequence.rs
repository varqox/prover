use std::{cell::RefCell, rc::Rc};

pub(crate) struct LazySequence<T> {
    pub(crate) vec: Rc<RefCell<Vec<T>>>,
}

pub(crate) struct LazySequenceIterator<T> {
    seq: Rc<RefCell<Vec<T>>>,
    next_idx: usize,
}

impl<T> LazySequence<T> {
    pub(crate) fn new(vec: Vec<T>) -> Self {
        Self {
            vec: Rc::new(RefCell::new(vec)),
        }
    }

    pub(crate) fn iter(&self) -> LazySequenceIterator<T> {
        LazySequenceIterator {
            seq: self.vec.clone(),
            next_idx: 0,
        }
    }
}

impl<'a, T: Clone> Iterator for LazySequenceIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let vec = self.seq.borrow_mut();
        if self.next_idx < vec.len() {
            let res = vec[self.next_idx].clone();
            self.next_idx += 1;
            Some(res)
        } else {
            None
        }
    }
}
