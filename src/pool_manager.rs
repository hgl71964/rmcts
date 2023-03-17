use crate::tree::ExpTask;

pub struct PoolManager {}

impl PoolManager {
    pub fn new() -> Self {
        PoolManager {}
    }

    pub fn wait_for_all(&self) {}

    pub fn kill_stragger(&mut self) {}

    pub fn has_idle_server(&mut self) -> bool {
        false
    }

    pub fn assign_expansion_task(
        &mut self,
        exp_task: ExpTask,
        global_saving_idx: u32,
        task_idx: u32,
    ) {
    }

    pub fn occupancy(&mut self) -> f32 {
        1.0
    }

    pub fn get_complete_expansion_task(&mut self) {}

    pub fn close(&mut self) {}
}
