use std::collections::HashMap;

use crate::{
    fol::{self, func_sig, skolemize, Fun, NameAllocator, Rel, Term},
    herbrand_universe::herbrand_universe,
    pl::{self, into_ecnf},
    pl_sat_solver::is_satisfiable,
    tuple_iterator::TupleIterator,
};

fn remove_universal_prefix(formula: fol::Formula) -> (Vec<fol::Var>, fol::Formula) {
    match formula {
        fol::Formula::Forall(var, phi) => {
            let (mut vars, formula) = remove_universal_prefix(*phi);
            vars.push(var);
            (vars, formula)
        }
        _ => (vec![], formula),
    }
}

#[derive(Default)]
struct RelToVar {
    rel_to_var: HashMap<(Rel, Vec<Term>), pl::Var>,
    var_alloc: pl::VarAllocator,
}

impl RelToVar {
    fn translate(&mut self, rel: Rel, terms: Vec<Term>) -> pl::Var {
        let alloc = &mut self.var_alloc;
        *self
            .rel_to_var
            .entry((rel, terms))
            .or_insert_with(|| alloc.alloc())
    }
}

pub(crate) fn is_tautology(formula: fol::Formula, fun_alloc: &mut NameAllocator<Fun>) -> bool {
    let formula = skolemize(fol::Formula::Not(Box::new(formula)), fun_alloc);
    let (vars, formula) = remove_universal_prefix(formula);

    fn interp_term(term: &Term, interp: &HashMap<fol::Var, Term>) -> Term {
        match term {
            Term::Var(var) => interp.get(&var).unwrap().clone(),
            Term::Fun(fun, args) => Term::Fun(
                *fun,
                args.into_iter()
                    .map(|term| interp_term(term, interp))
                    .collect(),
            ),
        }
    }

    fn into_pl_formula(
        formula: &fol::Formula,
        interp: &HashMap<fol::Var, Term>,
        rel_to_var: &mut RelToVar,
    ) -> pl::Formula {
        match formula {
            fol::Formula::True => pl::Formula::True,
            fol::Formula::False => pl::Formula::False,
            fol::Formula::Rel(rel, terms) => pl::Formula::Var(
                rel_to_var.translate(
                    *rel,
                    terms
                        .into_iter()
                        .map(|term| interp_term(term, interp))
                        .collect(),
                ),
            ),
            fol::Formula::Not(phi) => match phi.as_ref() {
                fol::Formula::Rel(rel, terms) => pl::Formula::NotVar(
                    rel_to_var.translate(
                        *rel,
                        terms
                            .into_iter()
                            .map(|term| interp_term(term, interp))
                            .collect(),
                    ),
                ),
                _ => panic!("expected NNF formula"),
            },
            fol::Formula::Or(a, b) => pl::Formula::Or(
                Box::new(into_pl_formula(a, interp, rel_to_var)),
                Box::new(into_pl_formula(b, interp, rel_to_var)),
            ),
            fol::Formula::And(a, b) => pl::Formula::And(
                Box::new(into_pl_formula(a, interp, rel_to_var)),
                Box::new(into_pl_formula(b, interp, rel_to_var)),
            ),
            _ => panic!("expected NNF formula"),
        }
    }

    let mut rel_to_var = RelToVar::default();
    let mut ecnf_prefix = pl::CNFFormula::new();
    let mut prefix_size = 0usize;
    let mut next_prefix_size_to_check = 2usize;
    let mut last_checked_prefix_size = 2usize;
    for var_terms in TupleIterator::new(herbrand_universe(func_sig(&formula)), vars.len()) {
        let pl_formula = into_pl_formula(
            &formula,
            &vars.iter().copied().zip(var_terms.into_iter()).collect(),
            &mut rel_to_var,
        );
        ecnf_prefix.extend(
            into_ecnf(pl_formula, &mut rel_to_var.var_alloc)
                .into_iter()
                .filter(|clause| !pl::clause_is_tautology(clause)),
        );

        prefix_size += 1;
        if prefix_size == next_prefix_size_to_check {
            if !is_satisfiable(ecnf_prefix.clone()) {
                return true;
            }
            if prefix_size / vars.len() > 1_000 {
                return false;
            }
            next_prefix_size_to_check = next_prefix_size_to_check * 3;
            last_checked_prefix_size = prefix_size;
        }
    }
    if prefix_size == last_checked_prefix_size {
        return false;
    }
    !is_satisfiable(ecnf_prefix)
}
