use std::collections::HashMap;

#[derive(Debug)]
pub struct CheckpointerManager {
    buffer: HashMap<u32, Vec<u32>>,
}

impl CheckpointerManager {
    pub fn new() -> Self {
        CheckpointerManager {
            buffer: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn checkpoint_env(&mut self, global_saving_idx: u32, action_history: Vec<u32>) {
        assert_eq!(self.buffer.get(&global_saving_idx), None);
        self.buffer.insert(global_saving_idx, action_history);
        // println!("[CheckpointerManager] checkpoint_env {:?}", self.buffer);
    }

    pub fn retrieve(&mut self, global_saving_idx: u32) -> Vec<u32> {
        assert_ne!(self.buffer.get(&global_saving_idx), None);
        let q = self.buffer.get(&global_saving_idx);
        q.unwrap().clone()
    }

    // pub fn restore(&mut self, global_saving_idx: u32, ) {
    // }

    pub fn load_checkpoint_env(&mut self) {}
}
