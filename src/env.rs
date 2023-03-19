use std::collections::HashMap;

pub struct Env {
    action_history: Vec<u32>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            action_history: Vec::new(),
        }
    }
    pub fn reset(&self) {}

    pub fn step(&self, action: usize) -> (u32, f32, bool, HashMap<u32, u32>) {
        (0, 0.0, true, HashMap::new())
    }

    pub fn get_action_space(&self) -> usize {
        2
    }

    pub fn checkpoint(&self) -> Vec<u32> {
        self.action_history.clone()
    }

    pub fn restore(&mut self, checkpoint_data: Vec<u32>) {}
}
