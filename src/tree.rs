#[path = "checkpoint_manager.rs"]
mod checkpoint_manager;
#[path = "env.rs"]
mod env;
#[path = "node.rs"]
mod node;
#[path = "pool_manager.rs"]
mod pool_manager;

pub struct Tree {
    budget: u32,
    max_sim_step: u32,
    gamma: f32,
    expansion_worker_num: u32,
    simulation_worker_num: u32,

    exp_pool: pool_manager::Pool_manager,
    sim_pool: pool_manager::Pool_manager,
    checkpoint_data_manager: checkpoint_manager::Checkpointer_manager,
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

            exp_pool: pool_manager::Pool_manager::new(),
            sim_pool: pool_manager::Pool_manager::new(),
            checkpoint_data_manager: checkpoint_manager::Checkpointer_manager::new(),
        }
    }

    pub fn plan(&self, state: u32) -> u32 {
        1
    }
}
