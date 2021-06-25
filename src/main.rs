use parser::formula::*;
use std::collections::HashMap;
use std::io::{self, Read};

use crate::prover::is_tautology;

mod fol;
mod herbrand_universe;
mod interleave;
mod lazy_sequence;
mod pl;
mod pl_sat_solver;
mod prover;
mod tuple_iterator;

fn parser_formula_to_fo_formula(
    pformula: Box<Formula>,
) -> (
    fol::Formula,
    fol::NameAllocator<fol::Var>,
    fol::NameAllocator<fol::Fun>,
    fol::NameAllocator<fol::Rel>,
) {
    #[derive(Default)]
    struct Translator {
        var_alloc: fol::NameAllocator<fol::Var>,
        vars: HashMap<String, fol::Var>,
        fun_alloc: fol::NameAllocator<fol::Fun>,
        funs: HashMap<String, fol::Fun>,
        rel_alloc: fol::NameAllocator<fol::Rel>,
        rels: HashMap<String, fol::Rel>,
    }

    impl Translator {
        fn translate_var(&mut self, var: String) -> fol::Var {
            let alloc = &mut self.var_alloc;
            *self.vars.entry(var).or_insert_with(|| alloc.alloc())
        }

        fn translate_fun(&mut self, fun: String) -> fol::Fun {
            let alloc = &mut self.fun_alloc;
            *self.funs.entry(fun).or_insert_with(|| alloc.alloc())
        }

        fn translate_rel(&mut self, rel: String) -> fol::Rel {
            let alloc = &mut self.rel_alloc;
            *self.rels.entry(rel).or_insert_with(|| alloc.alloc())
        }

        fn translate_term(&mut self, term: Term) -> fol::Term {
            match term {
                Term::Var(name) => fol::Term::Var(self.translate_var(name)),
                Term::Fun(name, terms) => fol::Term::Fun(
                    self.translate_fun(name),
                    terms.into_iter().map(|x| self.translate_term(x)).collect(),
                ),
            }
        }

        fn translate_formula(&mut self, formula: Formula) -> fol::Formula {
            match formula {
                Formula::True => fol::Formula::True,
                Formula::False => fol::Formula::False,
                Formula::Rel(name, terms) => fol::Formula::Rel(
                    self.translate_rel(name),
                    terms.into_iter().map(|x| self.translate_term(x)).collect(),
                ),
                Formula::Not(phi) => fol::Formula::Not(Box::new(self.translate_formula(*phi))),
                Formula::Or(phi, psi) => fol::Formula::Or(
                    Box::new(self.translate_formula(*phi)),
                    Box::new(self.translate_formula(*psi)),
                ),
                Formula::And(phi, psi) => fol::Formula::And(
                    Box::new(self.translate_formula(*phi)),
                    Box::new(self.translate_formula(*psi)),
                ),
                Formula::Implies(phi, psi) => fol::Formula::Implies(
                    Box::new(self.translate_formula(*phi)),
                    Box::new(self.translate_formula(*psi)),
                ),
                Formula::Iff(phi, psi) => fol::Formula::Iff(
                    Box::new(self.translate_formula(*phi)),
                    Box::new(self.translate_formula(*psi)),
                ),
                Formula::Exists(var, phi) => {
                    let old_mapping = self.vars.remove_entry(&var);
                    let res = fol::Formula::Exists(
                        self.translate_var(var.clone()),
                        Box::new(self.translate_formula(*phi)),
                    );
                    match old_mapping {
                        Some((k, v)) => {
                            self.vars.insert(k, v);
                        }
                        None => {
                            self.vars.remove(&var);
                        }
                    };
                    res
                }
                Formula::Forall(var, phi) => {
                    let old_mapping = self.vars.remove_entry(&var);
                    let res = fol::Formula::Forall(
                        self.translate_var(var.clone()),
                        Box::new(self.translate_formula(*phi)),
                    );
                    match old_mapping {
                        Some((k, v)) => {
                            self.vars.insert(k, v);
                        }
                        None => {
                            self.vars.remove(&var);
                        }
                    };
                    res
                }
            }
        }
    }

    let mut translator = Translator::default();
    (
        translator.translate_formula(*pformula),
        translator.var_alloc,
        translator.fun_alloc,
        translator.rel_alloc,
    )
}

fn main() {
    let mut raw_formula = String::new();
    io::stdin().read_to_string(&mut raw_formula).unwrap();
    let formula = parse_formula(&raw_formula).unwrap();
    let (formula, _var_alloc, mut fun_alloc, _rel_alloc) = parser_formula_to_fo_formula(formula);
    println!("{}", is_tautology(formula, &mut fun_alloc) as u8);
}
