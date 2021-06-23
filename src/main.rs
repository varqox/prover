use parser::formula::*;
use std::collections::HashMap;
use std::io::{self, Read};

use crate::fo::skolemize;

mod fo;

fn parser_formula_to_fo_formula(
    pformula: Box<Formula>,
) -> (
    fo::Formula,
    fo::NameAllocator<fo::Var>,
    fo::NameAllocator<fo::Fun>,
    fo::NameAllocator<fo::Rel>,
) {
    #[derive(Default)]
    struct Translator {
        var_alloc: fo::NameAllocator<fo::Var>,
        vars: HashMap<String, fo::Var>,
        fun_alloc: fo::NameAllocator<fo::Fun>,
        funs: HashMap<String, fo::Fun>,
        rel_alloc: fo::NameAllocator<fo::Rel>,
        rels: HashMap<String, fo::Rel>,
    }

    impl Translator {
        fn translate_var(&mut self, var: String) -> fo::Var {
            let alloc = &mut self.var_alloc;
            *self.vars.entry(var).or_insert_with(|| alloc.alloc())
        }

        fn translate_fun(&mut self, fun: String) -> fo::Fun {
            let alloc = &mut self.fun_alloc;
            *self.funs.entry(fun).or_insert_with(|| alloc.alloc())
        }

        fn translate_rel(&mut self, rel: String) -> fo::Rel {
            let alloc = &mut self.rel_alloc;
            *self.rels.entry(rel).or_insert_with(|| alloc.alloc())
        }

        fn translate_term(&mut self, term: Term) -> fo::Term {
            match term {
                Term::Var(name) => fo::Term::Var(self.translate_var(name)),
                Term::Fun(name, terms) => fo::Term::Fun(
                    self.translate_fun(name),
                    terms.into_iter().map(|x| self.translate_term(x)).collect(),
                ),
            }
        }

        fn translate_formula(&mut self, formula: Formula) -> fo::Formula {
            match formula {
                Formula::True => fo::Formula::True,
                Formula::False => fo::Formula::False,
                Formula::Rel(name, terms) => fo::Formula::Rel(
                    self.translate_rel(name),
                    terms.into_iter().map(|x| self.translate_term(x)).collect(),
                ),
                Formula::Not(phi) => fo::Formula::Not(Box::new(self.translate_formula(*phi))),
                Formula::Or(phi, psi) => fo::Formula::Or(
                    Box::new(self.translate_formula(*phi)),
                    Box::new(self.translate_formula(*psi)),
                ),
                Formula::And(phi, psi) => fo::Formula::And(
                    Box::new(self.translate_formula(*phi)),
                    Box::new(self.translate_formula(*psi)),
                ),
                Formula::Implies(phi, psi) => fo::Formula::Implies(
                    Box::new(self.translate_formula(*phi)),
                    Box::new(self.translate_formula(*psi)),
                ),
                Formula::Iff(phi, psi) => fo::Formula::Iff(
                    Box::new(self.translate_formula(*phi)),
                    Box::new(self.translate_formula(*psi)),
                ),
                Formula::Exists(var, phi) => {
                    let old_mapping = self.vars.remove_entry(&var);
                    let res = fo::Formula::Exists(
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
                    let res = fo::Formula::Forall(
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
    println!("{:?}", formula);
    let (fo_formula, _var_alloc, mut fun_alloc, _rel_alloc) = parser_formula_to_fo_formula(formula);
    println!("{:?}", fo_formula);
    let skolemized_formula = skolemize(fo_formula, &mut fun_alloc);
    println!("{:?}", skolemized_formula);
}
