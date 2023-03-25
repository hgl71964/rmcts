mod checkpoint_manager;
mod env;
mod eg_env;
mod node;
mod pool_manager;
mod run;
mod tree;
mod workers;
use egg::*;

define_language! {
    enum SimpleLanguage {
        Num(i32),
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        Symbol(Symbol),
    }
}

fn make_rules() -> Vec<Rewrite<SimpleLanguage, ()>> {
    vec![
        rewrite!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rewrite!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
        rewrite!("add-0"; "(+ ?a 0)" => "?a"),
        rewrite!("mul-0"; "(* ?a 0)" => "0"),
        rewrite!("mul-1"; "(* ?a 1)" => "?a"),
    ]
}

fn main() {
    run::run_mcts("(* 0 42)".parse().unwrap(), make_rules(), None);
}
