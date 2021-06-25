use std::collections::{BTreeSet, HashSet};

use itertools::Itertools;

use crate::fol;

pub(crate) type Var = fol::Var;

pub(crate) type VarAllocator = fol::NameAllocator<Var>;

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub(crate) enum Formula {
    True,
    False,
    Var(Var),
    NotVar(Var),
    And(Box<Formula>, Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, PartialOrd, Ord)]
pub(crate) enum Literal {
    Pos(Var),
    Neg(Var),
}

pub(crate) fn neg(lit: Literal) -> Literal {
    match lit {
        Literal::Pos(var) => Literal::Neg(var),
        Literal::Neg(var) => Literal::Pos(var),
    }
}

pub(crate) type CNFClause = BTreeSet<Literal>;
pub(crate) type CNFFormula = HashSet<CNFClause>;

fn into_cnf(formula: Formula) -> CNFFormula {
    match formula {
        Formula::True => HashSet::new(),
        Formula::False => vec![vec![].into_iter().collect()].into_iter().collect(),
        Formula::Var(var) => vec![vec![Literal::Pos(var)].into_iter().collect()]
            .into_iter()
            .collect(),
        Formula::NotVar(var) => vec![vec![Literal::Neg(var)].into_iter().collect()]
            .into_iter()
            .collect(),
        Formula::And(a, b) => into_cnf(*a)
            .into_iter()
            .chain(into_cnf(*b).into_iter())
            .collect(),
        Formula::Or(a, b) => into_cnf(*a)
            .iter()
            .cartesian_product(into_cnf(*b).iter())
            .map(|(a, b)| a.into_iter().chain(b.into_iter()).cloned().collect())
            .collect(),
    }
}

fn without_inner_true_false(formula: Formula) -> Formula {
    match formula {
        Formula::True => Formula::True,
        Formula::False => Formula::False,
        Formula::Var(var) => Formula::Var(var),
        Formula::NotVar(var) => Formula::NotVar(var),
        Formula::And(a, b) => match (without_inner_true_false(*a), without_inner_true_false(*b)) {
            (Formula::False, _) => Formula::False,
            (_, Formula::False) => Formula::False,
            (Formula::True, b) => b,
            (a, Formula::True) => a,
            (a, b) => Formula::And(Box::new(a), Box::new(b)),
        },
        Formula::Or(a, b) => match (without_inner_true_false(*a), without_inner_true_false(*b)) {
            (Formula::True, _) => Formula::True,
            (_, Formula::True) => Formula::True,
            (Formula::False, b) => b,
            (a, Formula::False) => a,
            (a, b) => Formula::Or(Box::new(a), Box::new(b)),
        },
    }
}

pub(crate) fn into_ecnf(formula: Formula, var_alloc: &mut VarAllocator) -> CNFFormula {
    fn tseitin(formula: Formula, var_alloc: &mut VarAllocator, res: &mut CNFFormula) -> Var {
        match formula {
            Formula::True | Formula::False => {
                panic!("should be removed by without_inner_true_false()")
            }
            Formula::Var(var) => var,
            Formula::NotVar(var) => {
                let v = var_alloc.alloc();
                res.extend(into_cnf(Formula::Or(
                    Box::new(Formula::And(
                        Box::new(Formula::Var(v)),
                        Box::new(Formula::NotVar(var)),
                    )),
                    Box::new(Formula::And(
                        Box::new(Formula::NotVar(v)),
                        Box::new(Formula::Var(var)),
                    )),
                )));
                v
            }
            Formula::And(a, b) => {
                let va = tseitin(*a, var_alloc, res);
                let vb = tseitin(*b, var_alloc, res);
                let v = var_alloc.alloc();
                res.extend(into_cnf(Formula::Or(
                    Box::new(Formula::And(
                        Box::new(Formula::Var(v)),
                        Box::new(Formula::And(
                            Box::new(Formula::Var(va)),
                            Box::new(Formula::Var(vb)),
                        )),
                    )),
                    Box::new(Formula::And(
                        Box::new(Formula::NotVar(v)),
                        Box::new(Formula::Or(
                            Box::new(Formula::NotVar(va)),
                            Box::new(Formula::NotVar(vb)),
                        )),
                    )),
                )));
                v
            }
            Formula::Or(a, b) => {
                let va = tseitin(*a, var_alloc, res);
                let vb = tseitin(*b, var_alloc, res);
                let v = var_alloc.alloc();
                res.extend(into_cnf(Formula::Or(
                    Box::new(Formula::And(
                        Box::new(Formula::Var(v)),
                        Box::new(Formula::Or(
                            Box::new(Formula::Var(va)),
                            Box::new(Formula::Var(vb)),
                        )),
                    )),
                    Box::new(Formula::And(
                        Box::new(Formula::NotVar(v)),
                        Box::new(Formula::And(
                            Box::new(Formula::NotVar(va)),
                            Box::new(Formula::NotVar(vb)),
                        )),
                    )),
                )));
                v
            }
        }
    }

    match without_inner_true_false(formula) {
        Formula::True => vec![].into_iter().collect(),
        Formula::False => vec![vec![].into_iter().collect()].into_iter().collect(),
        phi => {
            let mut res = CNFFormula::new();
            let v = tseitin(phi, var_alloc, &mut res);
            res.insert(vec![Literal::Pos(v)].into_iter().collect());
            res
        }
    }
}

pub(crate) fn clause_is_tautology(clause: &CNFClause) -> bool {
    for lit in clause {
        if clause.contains(&neg(*lit)) {
            return true;
        }
    }
    false
}
