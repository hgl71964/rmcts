use crate::tree;
use egg::{Analysis, Language, Rewrite};

pub fn run_mcts<L: Language, N: Analysis<L> + Clone>(expr: &str, rules: Vec<Rewrite<L, N>>) {
    let budget = 10;
    let max_sim_step = 5;
    let gamma = 1.0;
    let expansion_worker_num = 1;
    let simulation_worker_num = 4;
    let verbose = false;

    // mcts
    let mut mcts = tree::Tree::new(
        budget,
        max_sim_step,
        gamma,
        expansion_worker_num,
        simulation_worker_num,
        expr,
        rules.clone(),
    );
    mcts.run_loop(expr, rules.clone());
}
