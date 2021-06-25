use std::{
    collections::{HashMap, HashSet},
    fmt, mem,
};

use itertools::Itertools;

#[derive(PartialEq, Eq, Clone, Copy, Hash, Default, PartialOrd, Ord)]
pub(crate) struct Var {
    name: usize,
}

impl fmt::Debug for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "v_{}", self.name)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Default)]
pub(crate) struct Fun {
    name: usize,
}

impl fmt::Debug for Fun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "f_{}", self.name)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Default)]
pub(crate) struct Rel {
    name: usize,
}

impl fmt::Debug for Rel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "R_{}", self.name)
    }
}

pub(crate) trait NextName {
    fn next_name(&self) -> Self;
}

impl NextName for Var {
    fn next_name(&self) -> Self {
        Self {
            name: self.name + 1,
        }
    }
}

impl NextName for Fun {
    fn next_name(&self) -> Self {
        Self {
            name: self.name + 1,
        }
    }
}

impl NextName for Rel {
    fn next_name(&self) -> Self {
        Self {
            name: self.name + 1,
        }
    }
}

#[derive(Default)]
pub(crate) struct NameAllocator<T> {
    next: T,
}

impl<T: NextName + Default + Clone> NameAllocator<T> {
    pub(crate) fn alloc(&mut self) -> T {
        let next = self.next.next_name();
        mem::replace(&mut self.next, next)
    }
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub(crate) enum Term {
    Var(Var),
    Fun(Fun, Vec<Term>),
}

impl fmt::Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Term::Var(var) => write!(f, "{:?}", var),
            Term::Fun(fun, args) => write!(f, "{:?}({:?})", fun, args.iter().format(", ")),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub(crate) enum Formula {
    True,
    False,
    Rel(Rel, Vec<Term>),
    Not(Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
    And(Box<Formula>, Box<Formula>),
    Implies(Box<Formula>, Box<Formula>),
    Iff(Box<Formula>, Box<Formula>),
    Exists(Var, Box<Formula>),
    Forall(Var, Box<Formula>),
}

impl fmt::Debug for Formula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Formula::True => write!(f, "True"),
            Formula::False => write!(f, "False"),
            Formula::Rel(rel, args) => write!(f, "{:?}({:?})", rel, args.iter().format(", ")),
            Formula::Not(phi) => write!(f, "Not({:?})", phi),
            Formula::Or(phi, psi) => write!(f, "Or({:?}, {:?})", phi, psi),
            Formula::And(phi, psi) => write!(f, "And({:?}, {:?})", phi, psi),
            Formula::Implies(phi, psi) => write!(f, "Implies({:?}, {:?})", phi, psi),
            Formula::Iff(phi, psi) => write!(f, "Iff({:?}, {:?})", phi, psi),
            Formula::Exists(var, phi) => write!(f, "Exists({:?}, {:?})", var, phi),
            Formula::Forall(var, phi) => write!(f, "Forall({:?}, {:?})", var, phi),
        }
    }
}

pub(crate) fn into_nnf(formula: Formula) -> (Formula, NameAllocator<Var>) {
    #[derive(Default)]
    struct Transformer {
        var_alloc: NameAllocator<Var>,
        vars: HashMap<Var, Var>,
    }

    impl Transformer {
        fn remap_var(&mut self, var: Var) -> Var {
            let alloc = &mut self.var_alloc;
            *self.vars.entry(var).or_insert_with(|| alloc.alloc())
        }

        fn with_shadowed_var<T, F: FnOnce(&mut Self) -> T>(&mut self, var: Var, fun: F) -> T {
            let old_mapping = self.vars.remove_entry(&var);
            let res = fun(self);
            match old_mapping {
                Some((k, v)) => {
                    self.vars.insert(k, v);
                }
                None => {
                    self.vars.remove(&var);
                }
            }
            res
        }

        fn remap_term(&mut self, term: Term) -> Term {
            match term {
                Term::Var(var) => Term::Var(self.remap_var(var)),
                Term::Fun(func, args) => Term::Fun(
                    func,
                    args.into_iter().map(|term| self.remap_term(term)).collect(),
                ),
            }
        }

        fn into_nnf(&mut self, formula: Formula) -> Formula {
            match formula {
                Formula::True => Formula::True,
                Formula::False => Formula::False,
                Formula::Rel(rel, terms) => Formula::Rel(
                    rel,
                    terms
                        .into_iter()
                        .map(|term| self.remap_term(term))
                        .collect(),
                ),
                Formula::Or(a, b) => {
                    Formula::Or(Box::new(self.into_nnf(*a)), Box::new(self.into_nnf(*b)))
                }
                Formula::And(a, b) => {
                    Formula::And(Box::new(self.into_nnf(*a)), Box::new(self.into_nnf(*b)))
                }
                Formula::Implies(a, b) => Formula::Or(
                    Box::new(self.into_nnf(Formula::Not(a))),
                    Box::new(self.into_nnf(*b)),
                ),
                Formula::Iff(a, b) => self.into_nnf(Formula::Or(
                    Box::new(Formula::And(a.clone(), b.clone())),
                    Box::new(Formula::And(
                        Box::new(Formula::Not(a)),
                        Box::new(Formula::Not(b)),
                    )),
                )),
                Formula::Exists(var, phi) => self.with_shadowed_var(var, |me| {
                    Formula::Exists(me.remap_var(var), Box::new(me.into_nnf(*phi)))
                }),
                Formula::Forall(var, phi) => self.with_shadowed_var(var, |me| {
                    Formula::Forall(me.remap_var(var), Box::new(me.into_nnf(*phi)))
                }),
                Formula::Not(phi) => match *phi {
                    Formula::True => Formula::False,
                    Formula::False => Formula::True,
                    Formula::Rel(rel, terms) => {
                        Formula::Not(Box::new(self.into_nnf(Formula::Rel(rel, terms))))
                    }
                    Formula::Or(a, b) => self.into_nnf(Formula::And(
                        Box::new(Formula::Not(a)),
                        Box::new(Formula::Not(b)),
                    )),
                    Formula::And(a, b) => self.into_nnf(Formula::Or(
                        Box::new(Formula::Not(a)),
                        Box::new(Formula::Not(b)),
                    )),
                    Formula::Implies(a, b) => {
                        self.into_nnf(Formula::And(a, Box::new(Formula::Not(b))))
                    }
                    Formula::Iff(a, b) => self.into_nnf(Formula::And(
                        Box::new(Formula::Or(a.clone(), b.clone())),
                        Box::new(Formula::Or(
                            Box::new(Formula::Not(a)),
                            Box::new(Formula::Not(b)),
                        )),
                    )),
                    Formula::Exists(var, phi) => self.with_shadowed_var(var, |me| {
                        Formula::Forall(me.remap_var(var), Box::new(me.into_nnf(Formula::Not(phi))))
                    }),
                    Formula::Forall(var, phi) => self.with_shadowed_var(var, |me| {
                        Formula::Exists(me.remap_var(var), Box::new(me.into_nnf(Formula::Not(phi))))
                    }),
                    Formula::Not(phi) => self.into_nnf(*phi),
                },
            }
        }
    }

    let mut transformer = Transformer::default();
    let formula = transformer.into_nnf(formula);
    (formula, transformer.var_alloc)
}

pub(crate) fn into_pnf(formula: Formula) -> Formula {
    enum Quantifier {
        Exists(Var),
        Forall(Var),
    }
    fn extract_quantifiers(
        formula: Formula,
        extracted_quantifiers: &mut Vec<Quantifier>,
    ) -> Formula {
        match formula {
            Formula::True => Formula::True,
            Formula::False => Formula::False,
            Formula::Rel(rel, terms) => Formula::Rel(rel, terms),
            Formula::Not(phi) => {
                Formula::Not(Box::new(extract_quantifiers(*phi, extracted_quantifiers)))
            }
            Formula::And(a, b) => Formula::And(
                Box::new(extract_quantifiers(*a, extracted_quantifiers)),
                Box::new(extract_quantifiers(*b, extracted_quantifiers)),
            ),
            Formula::Or(a, b) => Formula::Or(
                Box::new(extract_quantifiers(*a, extracted_quantifiers)),
                Box::new(extract_quantifiers(*b, extracted_quantifiers)),
            ),
            Formula::Exists(var, phi) => {
                extracted_quantifiers.push(Quantifier::Exists(var));
                extract_quantifiers(*phi, extracted_quantifiers)
            }
            Formula::Forall(var, phi) => {
                extracted_quantifiers.push(Quantifier::Forall(var));
                extract_quantifiers(*phi, extracted_quantifiers)
            }
            _ => panic!("expected nnf formula"),
        }
    }
    let mut extracted_quantifiers = Vec::new();
    let formula = extract_quantifiers(into_nnf(formula).0, &mut extracted_quantifiers);
    extracted_quantifiers
        .into_iter()
        .rev()
        .fold(formula, |formula, quantifier| match quantifier {
            Quantifier::Exists(var) => Formula::Exists(var, Box::new(formula)),
            Quantifier::Forall(var) => Formula::Forall(var, Box::new(formula)),
        })
}

pub(crate) fn free_variables(formula: &Formula) -> HashSet<Var> {
    fn dfs_term(term: &Term, free_variables: &mut HashSet<Var>) {
        match term {
            Term::Var(var) => {
                free_variables.insert(*var);
            }
            Term::Fun(_, args) => {
                for term in args {
                    dfs_term(term, free_variables);
                }
            }
        }
    }
    fn dfs(formula: &Formula, free_variables: &mut HashSet<Var>) {
        match formula {
            Formula::True | Formula::False => {}
            Formula::Rel(_, terms) => {
                for term in terms {
                    dfs_term(term, free_variables)
                }
            }
            Formula::Not(phi) => dfs(phi, free_variables),
            Formula::Or(a, b) => {
                dfs(a, free_variables);
                dfs(b, free_variables)
            }
            Formula::And(a, b) => {
                dfs(a, free_variables);
                dfs(b, free_variables)
            }
            Formula::Implies(a, b) => {
                dfs(a, free_variables);
                dfs(b, free_variables)
            }
            Formula::Iff(a, b) => {
                dfs(a, free_variables);
                dfs(b, free_variables)
            }
            Formula::Exists(var, phi) => {
                dfs(phi, free_variables);
                free_variables.remove(var);
            }
            Formula::Forall(var, phi) => {
                dfs(phi, free_variables);
                free_variables.remove(var);
            }
        }
    }
    let mut fv = HashSet::new();
    dfs(formula, &mut fv);
    fv
}

pub(crate) fn into_sentence(formula: Formula) -> Formula {
    let fv = free_variables(&formula);
    fv.into_iter().fold(formula, |formula, var| {
        Formula::Forall(var, Box::new(formula))
    })
}

pub(crate) fn skolemize(formula: Formula, fun_alloc: &mut NameAllocator<Fun>) -> Formula {
    struct Skolemizer<'a> {
        env: Vec<Var>,
        varmap: HashMap<Var, Term>,
        fun_alloc: &'a mut NameAllocator<Fun>,
    }
    impl<'a> Skolemizer<'a> {
        fn skolemize_term(&mut self, term: Term) -> Term {
            match term {
                Term::Var(var) => self.varmap.get(&var).unwrap().clone(),
                Term::Fun(fun, args) => Term::Fun(
                    fun,
                    args.into_iter()
                        .map(|term| self.skolemize_term(term))
                        .collect(),
                ),
            }
        }

        fn skolemize(&mut self, formula: Formula) -> Formula {
            match formula {
                Formula::True => Formula::True,
                Formula::False => Formula::False,
                Formula::Rel(rel, terms) => Formula::Rel(
                    rel,
                    terms
                        .into_iter()
                        .map(|term| self.skolemize_term(term))
                        .collect(),
                ),
                Formula::Not(phi) => Formula::Not(Box::new(self.skolemize(*phi))),
                Formula::Or(a, b) => {
                    Formula::Or(Box::new(self.skolemize(*a)), Box::new(self.skolemize(*b)))
                }
                Formula::And(a, b) => {
                    Formula::And(Box::new(self.skolemize(*a)), Box::new(self.skolemize(*b)))
                }
                Formula::Implies(a, b) => {
                    Formula::Implies(Box::new(self.skolemize(*a)), Box::new(self.skolemize(*b)))
                }
                Formula::Iff(a, b) => {
                    Formula::Iff(Box::new(self.skolemize(*a)), Box::new(self.skolemize(*b)))
                }
                Formula::Exists(var, phi) => {
                    let check = self.varmap.insert(
                        var,
                        Term::Fun(
                            self.fun_alloc.alloc(),
                            self.env.iter().map(|x| Term::Var(*x)).collect(),
                        ),
                    );
                    assert!(check.is_none());
                    let res = self.skolemize(*phi);
                    self.varmap.remove(&var);
                    res
                }
                Formula::Forall(var, phi) => {
                    self.env.push(var);
                    let check = self.varmap.insert(var, Term::Var(var));
                    assert!(check.is_none());
                    let res = Formula::Forall(var, Box::new(self.skolemize(*phi)));
                    self.env.pop();
                    self.varmap.remove(&var);
                    res
                }
            }
        }
    }
    into_pnf(
        Skolemizer {
            env: Vec::new(),
            varmap: HashMap::new(),
            fun_alloc,
        }
        .skolemize(into_nnf(into_sentence(formula)).0),
    )
}

pub(crate) fn func_sig(formula: &Formula) -> HashSet<(Fun, usize)> {
    fn dfs_term(term: &Term, func_sig: &mut HashSet<(Fun, usize)>) {
        match term {
            Term::Var(_) => {}
            Term::Fun(fun, args) => {
                func_sig.insert((*fun, args.len()));
                for arg in args {
                    dfs_term(arg, func_sig);
                }
            }
        }
    }
    fn dfs(formula: &Formula, func_sig: &mut HashSet<(Fun, usize)>) {
        match formula {
            Formula::True | Formula::False => {}
            Formula::Rel(_, terms) => {
                for term in terms {
                    dfs_term(term, func_sig);
                }
            }
            Formula::Not(phi) => dfs(phi, func_sig),
            Formula::Or(a, b) => {
                dfs(a, func_sig);
                dfs(b, func_sig);
            }
            Formula::And(a, b) => {
                dfs(a, func_sig);
                dfs(b, func_sig);
            }
            Formula::Implies(a, b) => {
                dfs(a, func_sig);
                dfs(b, func_sig);
            }
            Formula::Iff(a, b) => {
                dfs(a, func_sig);
                dfs(b, func_sig);
            }
            Formula::Exists(_, phi) => dfs(phi, func_sig),
            Formula::Forall(_, phi) => dfs(phi, func_sig),
        }
    }

    let mut fs = HashSet::new();
    dfs(formula, &mut fs);
    fs
}
