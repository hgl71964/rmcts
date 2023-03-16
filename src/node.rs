pub struct Node {
    action_n: u32,
    checkpoint_idx: u32,
    parent: Option<Box<Node>>,
    gamma: f32,
    is_head: bool,
}

impl Node {
    pub fn new(action_n: u32, checkpoint_idx: u32, gamma: f32, is_head: bool) -> Self {
        Node {
            action_n: action_n,
            checkpoint_idx: checkpoint_idx,
            parent: None,
            gamma: gamma,
            is_head: is_head,
        }
    }

    pub fn default() -> Self {
        Node {
            action_n: 0,
            checkpoint_idx: 0,
            parent: None,
            gamma: 1.0,
            is_head: false,
        }
    }

    pub fn all_child_visited(&self) -> bool {
        true
    }

    pub fn shallow_clone(&self) -> bool {
        true
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.checkpoint_idx == other.checkpoint_idx
    }

impl Eq for Node {}

