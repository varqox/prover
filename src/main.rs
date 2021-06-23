use std::io::{self, Read};
use parser::formula::*;

fn main() {
    let mut raw_formula = String::new();
    io::stdin().read_to_string(&mut raw_formula).unwrap();
    let formula = parse_formula(&raw_formula).unwrap();
    println!("{:?}", formula);
}
