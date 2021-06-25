use std::collections::{BTreeSet, HashMap, HashSet};

use crate::pl::{clause_is_tautology, CNFClause, CNFFormula, Literal, Var};

pub(crate) fn is_satisfiable(formula: CNFFormula) -> bool {
    let formula: HashMap<_, _> = formula
        .into_iter()
        .filter(|clause| !clause_is_tautology(clause))
        .enumerate()
        .collect();

    #[derive(Default, Clone, Debug)]
    struct VarOccurs {
        pos: HashSet<usize>, // clause idx
        neg: HashSet<usize>, // clause idx
    }

    #[derive(Debug, Clone)]
    struct Solver {
        clauses: HashMap<usize, CNFClause>,
        var_occurs: HashMap<Var, VarOccurs>,
        var_rank: BTreeSet<(usize, Var)>,
        len_to_clause: HashMap<usize, HashSet<usize>>, // clause len => clause idx
        pure_vars: Vec<Var>,
    }

    impl Solver {
        fn delete_clause(&mut self, clause_idx: usize, deleted_var: Var) {
            let clause = self.clauses.remove(&clause_idx).unwrap();
            {
                let set = self.len_to_clause.get_mut(&clause.len()).unwrap();
                assert!(set.remove(&clause_idx));
                if set.is_empty() {
                    self.len_to_clause.remove(&clause.len());
                }
            }
            for literal in clause {
                match literal {
                    Literal::Pos(var) => {
                        if var != deleted_var {
                            let var_occurs = self.var_occurs.get_mut(&var).unwrap();
                            assert!(self
                                .var_rank
                                .remove(&(var_occurs.pos.len() + var_occurs.neg.len(), var)));
                            assert!(var_occurs.pos.remove(&clause_idx));
                            self.var_rank
                                .insert((var_occurs.pos.len() + var_occurs.neg.len(), var));
                            if var_occurs.pos.is_empty() && !var_occurs.neg.is_empty() {
                                self.pure_vars.push(var);
                            }
                        }
                    }
                    Literal::Neg(var) => {
                        if var != deleted_var {
                            let var_occurs = self.var_occurs.get_mut(&var).unwrap();
                            assert!(self
                                .var_rank
                                .remove(&(var_occurs.pos.len() + var_occurs.neg.len(), var)));
                            assert!(var_occurs.neg.remove(&clause_idx));
                            self.var_rank
                                .insert((var_occurs.pos.len() + var_occurs.neg.len(), var));
                            if var_occurs.neg.is_empty() && !var_occurs.pos.is_empty() {
                                self.pure_vars.push(var);
                            }
                        }
                    }
                }
            }
        }

        fn substitute_var(&mut self, var: Var, value: bool) {
            let VarOccurs { pos, neg } = self.var_occurs.remove(&var).unwrap();
            assert!(self.var_rank.remove(&(pos.len() + neg.len(), var)));
            let (to_delete, to_simplify) = if value { (pos, neg) } else { (neg, pos) };
            for clause_idx in to_delete {
                self.delete_clause(clause_idx, var);
            }
            for clause_idx in to_simplify {
                let literal_to_delete = if value {
                    Literal::Neg(var)
                } else {
                    Literal::Pos(var)
                };
                let clause = self.clauses.get_mut(&clause_idx).unwrap();
                assert!(clause.remove(&literal_to_delete));
                {
                    let old_clause_len = clause.len() + 1;
                    let set = self.len_to_clause.get_mut(&old_clause_len).unwrap();
                    assert!(set.remove(&clause_idx));
                    if set.is_empty() {
                        self.len_to_clause.remove(&old_clause_len);
                    }
                }
                self.len_to_clause
                    .entry(clause.len())
                    .or_insert(HashSet::new())
                    .insert(clause_idx);
            }
        }

        fn is_satisfiable(&mut self) -> bool {
            if self.len_to_clause.contains_key(&0) {
                return false;
            }
            while !self.pure_vars.is_empty() || self.len_to_clause.contains_key(&1) {
                while !self.pure_vars.is_empty() {
                    let pure_var = self.pure_vars.pop().unwrap();
                    let VarOccurs { pos, neg } = self.var_occurs.remove(&pure_var).unwrap();
                    assert!(self.var_rank.remove(&(pos.len() + neg.len(), pure_var)));
                    for &clause_idx in [pos, neg].iter().flatten() {
                        self.delete_clause(clause_idx, pure_var);
                    }
                }

                if let Some(unit_literal) = self.len_to_clause.get(&1).map(|unit_clauses| {
                    self.clauses
                        .get(unit_clauses.iter().next().unwrap())
                        .unwrap()
                        .iter()
                        .next()
                        .unwrap()
                        .clone()
                }) {
                    match unit_literal {
                        Literal::Pos(var) => self.substitute_var(var, true),
                        Literal::Neg(var) => self.substitute_var(var, false),
                    }
                    if self.len_to_clause.contains_key(&0) {
                        return false;
                    }
                }
            }

            if self.len_to_clause.is_empty() {
                return true;
            }
            // Now we have to make a guess
            let mut self_clone = self.clone();
            assert_eq!(self.var_occurs.len(), self.var_rank.len());
            let some_var = self.var_rank.iter().next_back().unwrap().1;
            // let some_var = *self.var_occurs.iter().next().unwrap().0;
            self_clone.substitute_var(some_var, true);
            if self_clone.is_satisfiable() {
                return true;
            }
            self.substitute_var(some_var, false);
            self.is_satisfiable()
        }
    }

    let mut var_occurs = HashMap::new();
    let mut len_to_clause = HashMap::new();
    for (&idx, clause) in formula.iter() {
        for literal in clause {
            match literal {
                Literal::Pos(var) => {
                    var_occurs
                        .entry(*var)
                        .or_insert(VarOccurs::default())
                        .pos
                        .insert(idx);
                }
                Literal::Neg(var) => {
                    var_occurs
                        .entry(*var)
                        .or_insert(VarOccurs::default())
                        .neg
                        .insert(idx);
                }
            }
        }
        len_to_clause
            .entry(clause.len())
            .or_insert(HashSet::new())
            .insert(idx);
    }

    let mut pure_vars = Vec::new();
    let mut var_rank = BTreeSet::new();
    for (&var, vo) in &var_occurs {
        if vo.pos.is_empty() || vo.neg.is_empty() {
            pure_vars.push(var);
        }
        var_rank.insert((vo.pos.len() + vo.neg.len(), var));
    }

    Solver {
        clauses: formula,
        var_rank,
        var_occurs,
        len_to_clause,
        pure_vars,
    }
    .is_satisfiable()
}
