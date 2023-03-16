use std::collections::HashMap;

pub struct Env {
    pub id: u32,
}

impl Env {
    pub fn reset(&self) {}

    pub fn step(&self) -> (u32, u32, bool, HashMap<u32, u32>) {
        (0, 0, true, HashMap::new())
    }

    pub fn checkpoint(&self) {}

    pub fn restore(&self) {}
}
