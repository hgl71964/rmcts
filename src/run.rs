#[path = "env.rs"]
mod env;
#[path = "tree.rs"]
mod tree;

use std::collections::HashMap;
use std::time::{Duration, Instant};

pub fn run() {
    run_loop();
}

fn run_loop() {
    // params
    let budget = 64;
    let max_sim_step = 5;
    let gamma = 1.0;
    let expansion_worker_num = 1;
    let simulation_worker_num = 4;

    // env
    let env = env::Env { id: 0 };

    // mcts
    let mcts = tree::Tree::new(
        budget,
        max_sim_step,
        gamma,
        expansion_worker_num,
        simulation_worker_num,
    );

    // loop var
    let mut state = 0;
    let mut reward = 0;
    let mut done = false;
    let mut info = HashMap::<u32, u32>::new();
    let mut cnt = 0;
    let mut episode_reward = 0;

    // env loop
    loop {
        let planning_time = Instant::now();
        let action = mcts.plan(state);
        let planning_time = planning_time.elapsed().as_secs();
        println!("planning time {}", planning_time);

        (state, reward, done, info) = env.step();

        cnt += 1;
        episode_reward += reward;

        println!(
            "iter: {} - reward: {} - cumulative_reward: {}",
            cnt, reward, episode_reward
        );

        if done {
            break;
        }
    }
}
