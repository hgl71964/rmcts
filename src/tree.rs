#[path = "checkpoint_manager.rs"]
mod checkpoint_manager;
#[path = "env.rs"]
mod env;
#[path = "node.rs"]
mod node;
#[path = "pool_manager.rs"]
mod pool_manager;

use env::Env;
use node::Node;
use rand::Rng;

use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

struct SimTask {}
struct ExpTask {}

pub struct Tree {
    // from param
    budget: u32,
    max_sim_step: u32,
    gamma: f32,
    expansion_worker_num: u32,
    simulation_worker_num: u32,

    exp_pool: pool_manager::PoolManager,
    sim_pool: pool_manager::PoolManager,
    checkpoint_data_manager: checkpoint_manager::CheckpointerManager,

    // for planning
    root_node: Node,
    global_saving_idx: u32,
    simulation_count: u32,
    expansion_task: HashMap<u32, ExpTask>,
    simulation_task: HashMap<u32, SimTask>,
    pending_expansion_task: HashMap<u32, ExpTask>,
    pending_simulation_task: HashMap<u32, SimTask>,
}

impl Tree {
    pub fn new(
        budget: u32,
        max_sim_step: u32,
        gamma: f32,
        expansion_worker_num: u32,
        simulation_worker_num: u32,
    ) -> Self {
        Tree {
            budget: budget,
            max_sim_step: max_sim_step,
            gamma: gamma,
            expansion_worker_num: expansion_worker_num,
            simulation_worker_num: simulation_worker_num,

            exp_pool: pool_manager::PoolManager::new(),
            sim_pool: pool_manager::PoolManager::new(),
            checkpoint_data_manager: checkpoint_manager::CheckpointerManager::new(),

            root_node: Node::default(),
            global_saving_idx: 0,
            simulation_count: 0,
            expansion_task: HashMap::new(),
            simulation_task: HashMap::new(),
            pending_expansion_task: HashMap::new(),
            pending_simulation_task: HashMap::new(),
        }
    }

    pub fn run_loop(&mut self) {
        // env
        let env = Env::new();

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
            let action = self.plan(&state, &env);
            let planning_time = planning_time.elapsed().as_secs();
            println!("planning time {}s", planning_time);

            (state, reward, done, info) = env.step(action);

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

    fn plan(&mut self, state: &u32, env: &Env) -> u32 {
        // skip if action space is 1
        let action_n = env.get_action_space();
        if action_n == 1 {
            return 0;
        }

        // clear
        self.global_saving_idx = 0;
        self.simulation_count = 0;
        self.checkpoint_data_manager.clear();
        self.expansion_task.clear();
        self.simulation_task.clear();
        self.pending_expansion_task.clear();
        self.pending_simulation_task.clear();
        self.exp_pool.wait_for_all();
        self.sim_pool.wait_for_all();

        // build current state
        self.checkpoint_data_manager
            .checkpoint_env(self.global_saving_idx, env.checkpoint());
        self.root_node = Node::new(action_n, self.global_saving_idx, self.gamma, true);
        self.global_saving_idx += 1;

        // run main mcts
        for sim_idx in 0..self.budget {
            self.simulate_single_step(&sim_idx);
        }

        //
        println!(
            "[WU-UCT] complete count {}/{} ",
            self.simulation_count, self.budget
        );
        thread::sleep(Duration::from_secs(1));

        self.exp_pool.kill_stragger();
        self.sim_pool.kill_stragger();

        // retrieve
        // self.checkpoint_data_manager.load_checkpoint_env(self.root_node.checkpoint_idx);

        self.max_action()
    }

    fn simulate_single_step(&mut self, sim_idx: &u32) {
        // selection
        let curr_node = &self.root_node;
        let mut curr_depth = 1;

        loop {
            break;
        }
    }

    fn max_action(&self) -> u32 {
        0
    }
}
