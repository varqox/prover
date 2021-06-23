// Author: Michał Niciejewski
use pest::Parser;
use pest::error::Error;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct FormulaParser;

#[derive(Hash, Eq, Debug, Clone)]
pub enum GenTerm<VarType> {
    Var(VarType),
    Fun(String, Vec<GenTerm<VarType>>),
}

impl<VarType: std::cmp::PartialEq> PartialEq for GenTerm<VarType> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (GenTerm::Var(name1), GenTerm::Var(name2)) => name1 == name2,
            (GenTerm::Fun(name1, terms1), GenTerm::Fun(name2, terms2)) => name1 == name2 && terms1 == terms2,
            _ => false
        }
    }
}

pub type Term = GenTerm<String>;

#[derive(Debug)]
pub enum Formula {
    False,
    True,
    Rel(String, Vec<Term>),
    Not(FormulaBox),
    Or(FormulaBox, FormulaBox),
    And(FormulaBox, FormulaBox),
    Implies(FormulaBox, FormulaBox),
    Iff(FormulaBox, FormulaBox),
    Exists(String, FormulaBox),
    Forall(String, FormulaBox),
}

pub type FormulaBox = Box<Formula>;

pub fn parse_formula(raw_formula: &str) -> Result<FormulaBox, Error<Rule>> {
    let parsed_formula = FormulaParser::parse(Rule::file, raw_formula)?.next().unwrap();

    use pest::iterators::Pair;

    fn build_ast_term(pair: Pair<Rule>) -> Term {
        let rule = pair.as_rule();
        let mut inner_rules = pair.into_inner();
        macro_rules! get_next_inner {
            () => { inner_rules.next().unwrap().into_inner(); };
        }
        macro_rules! parse_next_string {
            () => { String::from(get_next_inner!().as_str()); };
        }
        match rule {
            Rule::t_var => Term::Var(parse_next_string!()),
            Rule::t_fun => Term::Fun(parse_next_string!(),
                get_next_inner!().map(|pair| { build_ast_term(pair) }).collect()),
            _ => unreachable!(),
        }
    }

    fn build_ast_formula(pair: Pair<Rule>) -> FormulaBox {
        let rule = pair.as_rule();
        let mut inner_rules = pair.into_inner();
        macro_rules! get_next_inner {
            () => { inner_rules.next().unwrap().into_inner(); };
        }
        macro_rules! parse_next_formula {
            () => { build_ast_formula(inner_rules.next().unwrap()); };
        }
        macro_rules! parse_next_string {
            () => { String::from(get_next_inner!().as_str()); };
        }

        let formula = match rule {
            Rule::f_rel => Formula::Rel(parse_next_string!(),
                get_next_inner!().map(|pair| { build_ast_term(pair) }).collect()),
            Rule::f_not => Formula::Not(parse_next_formula!()),
            Rule::f_and => Formula::And(parse_next_formula!(), parse_next_formula!()),
            Rule::f_or => Formula::Or(parse_next_formula!(), parse_next_formula!()),
            Rule::f_implies => Formula::Implies(parse_next_formula!(), parse_next_formula!()),
            Rule::f_iff => Formula::Iff(parse_next_formula!(), parse_next_formula!()),
            Rule::f_forall => Formula::Forall(parse_next_string!(), parse_next_formula!()),
            Rule::f_exists => Formula::Exists(parse_next_string!(), parse_next_formula!()),
            Rule::f_true => Formula::True,
            Rule::f_false => Formula::False,
            _ => unreachable!(),
        };
        FormulaBox::new(formula)
    }

    Ok(build_ast_formula(parsed_formula))
}
