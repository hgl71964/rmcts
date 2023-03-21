use egg::{Analysis, EGraph, Language, Rewrite, Runner};
use std::collections::HashMap;

pub struct Env<L, N> {
    action_history: Vec<u32>,
    expr: &'static str,
    rules: Vec<Rewrite<L, N>>,
}

impl<L: Language, N: Analysis<L> + Clone> Env<L, N> {
    pub fn new(expr: &'static str, rules: Vec<Rewrite<L, N>>) -> Self {
        Env {
            action_history: Vec::new(),
            expr: expr,
            rules: rules,
        }
    }
    pub fn reset(&mut self) {}

    pub fn step(&mut self, action: usize) -> (u32, f32, bool, HashMap<u32, u32>) {
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
