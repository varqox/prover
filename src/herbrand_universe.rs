use std::collections::{BTreeMap, HashSet};

use crate::{
    fo::{Fun, Term},
    interleave::Interleave,
    lazy_sequence::LazySequence,
    tuple_iterator::TupleIterator,
};

pub(crate) struct HerbrandUniverse<I: Iterator<Item = Term>> {
    elems: LazySequence<Term>,
    higher_arity_terms_iter: I,
    next_idx: usize,
}

pub(crate) fn herbrand_universe(
    mut func_sig: HashSet<(Fun, usize)>,
) -> HerbrandUniverse<Box<dyn Iterator<Item = Term>>> {
    let mut exists_func_with_arity_0 = false;
    for (_, arity) in &func_sig {
        if *arity == 0 {
            exists_func_with_arity_0 = true;
        }
    }
    if !exists_func_with_arity_0 {
        func_sig.insert((Fun::default(), 0));
    }
    let mut res = BTreeMap::<_, HashSet<_>>::new();
    for (fun, arity) in func_sig.into_iter() {
        res.entry(arity).or_default().insert(fun);
    }
    let func_sig = res;
    let constants = LazySequence::new(
        func_sig
            .get(&0)
            .unwrap()
            .iter()
            .map(|fun| Term::Fun(*fun, Vec::new()))
            .collect(),
    );

    let mut higher_arity_terms_iterators = Vec::new();
    for (arity, funs) in func_sig.iter().skip(1) {
        for fun in funs {
            let fun = *fun;
            let term_iter =
                TupleIterator::new(constants.iter(), *arity).map(move |args| Term::Fun(fun, args));
            higher_arity_terms_iterators.push((term_iter, arity * arity));
        }
    }
    let higher_arity_terms_iter = Interleave::new(higher_arity_terms_iterators);

    HerbrandUniverse {
        elems: constants,
        higher_arity_terms_iter: Box::new(higher_arity_terms_iter),
        next_idx: 0,
    }
}

impl<T: Iterator<Item = Term>> Iterator for HerbrandUniverse<T> {
    type Item = Term;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_idx < self.elems.vec.borrow().len() {
            let elem = self.elems.vec.borrow()[self.next_idx].clone();
            self.next_idx += 1;
            Some(elem)
        } else {
            match self.higher_arity_terms_iter.next() {
                Some(elem) => {
                    self.elems.vec.borrow_mut().push(elem.clone());
                    self.next_idx += 1;
                    Some(elem)
                }
                None => None,
            }
        }
    }
}
