use crate::tree;
use egg::{Analysis, Language, RecExpr, Rewrite};

pub struct MCTSArgs {
    pub budget: u32,
    pub max_sim_step: u32,
    pub gamma: f32,
    pub expansion_worker_num: usize,
    pub simulation_worker_num: usize,

    pub node_limit: usize,
    pub time_limit: usize,
}

pub fn run_mcts<
    L: Language + 'static + egg::FromOp + std::marker::Send,
    N: Analysis<L> + Clone + 'static + std::default::Default + std::marker::Send,
>(
    expr: RecExpr<L>,
    rules: Vec<Rewrite<L, N>>,
    args: Option<MCTSArgs>,
) {
    // Args
    // mcts
    let mut budget = 12;
    let mut max_sim_step = 5;
    let mut gamma = 0.99;
    let mut expansion_worker_num = 1;
    let mut simulation_worker_num = 4;
    // let verbose = false;
    // egg
    let mut node_limit = 10_000;
    let mut time_limit = 1;

    // overwrite params if possible
    match args {
        None => (),
        Some(args) => {
            budget = args.budget;
            max_sim_step = args.max_sim_step;
            gamma = args.gamma;
            expansion_worker_num = args.expansion_worker_num;
            simulation_worker_num = args.simulation_worker_num;

            node_limit = args.node_limit;
            time_limit = args.time_limit;
        }
    }

    // Run
    let mut mcts = tree::Tree::new(
        // mcts
        budget,
        max_sim_step,
        gamma,
        expansion_worker_num,
        simulation_worker_num,
        // egg
        expr.clone(),
        rules.clone(),
        node_limit,
        time_limit,
    );
    mcts.run_loop(expr, rules.clone());
}
