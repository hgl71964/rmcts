use crate::checkpoint_manager;
use crate::env::Env;
use crate::node::{Node, NodeStub};
use crate::pool_manager;

use rand::Rng;
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

struct ExpTask {
    ckpt: Vec<u32>,
    shallow_copy_node: NodeStub,
}
struct SimTask {}

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
    expansion_tasks: HashMap<u32, ExpTask>,
    simulation_task: HashMap<u32, SimTask>,
    pending_expansion_tasks: Vec<u32>,
    pending_simulation_tasks: Vec<u32>,
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
            expansion_tasks: HashMap::new(),
            simulation_task: HashMap::new(),
            pending_expansion_tasks: Vec::new(),
            pending_simulation_tasks: Vec::new(),
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
        self.expansion_tasks.clear();
        self.simulation_task.clear();
        self.pending_expansion_tasks.clear();
        self.pending_simulation_tasks.clear();
        self.exp_pool.wait_for_all();
        self.sim_pool.wait_for_all();

        // build current state
        self.checkpoint_data_manager
            .checkpoint_env(self.global_saving_idx, env.checkpoint());
        self.root_node = Node::new(action_n, self.global_saving_idx, self.gamma, true);
        self.global_saving_idx += 1;

        // run main mcts
        for sim_idx in 0..self.budget {
            self.simulate_single_step(sim_idx);
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

        // debug!!
        // let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        // let v2: Vec<_> = v.iter().filter_map(|&x| if x % 2 == 0 { Some(x) } else { None }).collect();
        // println!("{:?}", v2);

        // let v3 = vec![Some(1), None];
        // let v4: Vec<_> = v3.iter().filter_map(|&x| if x.is_some() {x} else {None}).collect();
        // println!("{:?}", v4);

        self.max_action()
    }

    fn simulate_single_step(&mut self, sim_idx: u32) {
        // selection
        let mut curr_node = &mut self.root_node;
        let mut curr_depth = 1;
        let mut rng = rand::thread_rng();
        let mut need_expansion = false;

        loop {
            let rand = rng.gen_range(0.0..1.0);
            if (curr_node.no_child_available())
                || ((!curr_node.all_child_visited()) && !curr_node.is_head && rand < 0.5)
                || ((!curr_node.all_child_visited()) && curr_node.is_head)
            {
                // If no child node has been updated, we have to expand anyway.
                // Or if root node is not fully visited.
                // Or if non-root node is not fully visited and {with prob 1/2}.

                let cloned_curr_node = curr_node.shallow_clone();
                let checkpoint_data = self
                    .checkpoint_data_manager
                    .retrieve(curr_node.checkpoint_idx);

                // Record the task
                self.expansion_tasks.insert(
                    sim_idx,
                    ExpTask {
                        ckpt: checkpoint_data,
                        shallow_copy_node: cloned_curr_node,
                    },
                );
                self.pending_expansion_tasks.push(sim_idx);

                need_expansion = true;
                break;
            }

            let action = curr_node.selection_action();
            curr_node.update_history(sim_idx, action, curr_node.rewards[action]);

            if curr_node.dones[action] {
                // exceed maximum depth
                need_expansion = false;
                break;
            }

            // one-level deeper
            curr_depth += 1;
            curr_node = &mut curr_node.children[action].unwrap();

            break; // safe guard
        }

        // expansion
    }

    fn max_action(&self) -> u32 {
        0
    }
}
