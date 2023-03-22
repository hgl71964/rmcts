use crate::tree;
use egg::{Analysis, Language, Rewrite};

pub fn run_mcts<
    L: Language + 'static + egg::FromOp,
    N: Analysis<L> + Clone + 'static + std::default::Default,
>(
    expr: &'static str,
    rules: Vec<Rewrite<L, N>>,
) {
    // mcts
    let budget = 10;
    let max_sim_step = 5;
    let gamma = 1.0;
    let expansion_worker_num = 1;
    let simulation_worker_num = 4;
    let verbose = false;
    // egg
    let node_limit = 10_000;
    let time_limit = 10;

    // mcts
    let mut mcts = tree::Tree::new(
        // mcts
        budget,
        max_sim_step,
        gamma,
        expansion_worker_num,
        simulation_worker_num,
        // egg
        expr,
        rules.clone(),
        node_limit,
        time_limit,
    );
    mcts.run_loop(expr, rules.clone());
}
