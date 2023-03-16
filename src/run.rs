#[path = "tree.rs"]
mod tree;

pub fn run() {
    let budget = 64;
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
    );
    mcts.run_loop();
}
