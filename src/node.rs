use rand::Rng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Node {
    pub action_n: u32,
    pub checkpoint_idx: u32,
    pub parent: Option<Rc<RefCell<Node>>>,
    pub gamma: f32,
    pub is_head: bool,

    // children
    pub children: Vec<Option<Rc<RefCell<Node>>>>,
    pub rewards: Vec<f32>,
    pub dones: Vec<bool>,
    children_visit_count: Vec<u32>,
    children_complete_visit_count: Vec<u32>,
    children_saturated: Vec<bool>,
    q_value: Vec<f32>,

    // self
    visit_count: u32,
    traverse_history: HashMap<u32, (usize, f32)>,
    visited_node_count: u32,
    updated_node_count: u32,
}

impl Node {
    pub fn new(action_n: u32, checkpoint_idx: u32, gamma: f32, is_head: bool) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            action_n: action_n,
            checkpoint_idx: checkpoint_idx,
            parent: None,
            gamma: gamma,
            is_head: is_head,
            children: vec![None; usize::try_from(action_n).unwrap()],
            rewards: vec![0.0; usize::try_from(action_n).unwrap()],
            dones: vec![false; usize::try_from(action_n).unwrap()],
            children_visit_count: vec![0; usize::try_from(action_n).unwrap()],
            children_complete_visit_count: vec![0; usize::try_from(action_n).unwrap()],
            children_saturated: vec![false; usize::try_from(action_n).unwrap()],
            q_value: vec![0.0; usize::try_from(action_n).unwrap()],
            visit_count: 0,
            traverse_history: HashMap::new(),
            visited_node_count: 0,
            updated_node_count: 0,
        }))
    }

    pub fn default() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            action_n: 0,
            checkpoint_idx: 0,
            parent: None,
            gamma: 1.0,
            is_head: false,
            children: vec![None; usize::try_from(1).unwrap()],
            rewards: vec![0.0; usize::try_from(1).unwrap()],
            dones: vec![false; usize::try_from(1).unwrap()],
            children_visit_count: vec![0; usize::try_from(1).unwrap()],
            children_complete_visit_count: vec![0; usize::try_from(1).unwrap()],
            children_saturated: vec![false; usize::try_from(1).unwrap()],
            q_value: vec![0.0; usize::try_from(1).unwrap()],
            visit_count: 0,
            traverse_history: HashMap::new(),
            visited_node_count: 0,
            updated_node_count: 0,
        }))
    }

    pub fn all_child_visited(&self) -> bool {
        self.visited_node_count == self.action_n
    }

    pub fn no_child_available(&self) -> bool {
        self.updated_node_count == 0
    }

    pub fn all_child_updated(&self) -> bool {
        self.updated_node_count == self.action_n
    }

    pub fn shallow_clone(&self) -> NodeStub {
        // NOTE: the stub only needs to know whether a child exists or not
        let children: Vec<u32> = self
            .children
            .iter()
            .map(|x| if x.is_some() { 1 } else { 0 })
            .collect();
        NodeStub {
            action_n: self.action_n,
            checkpoint_idx: self.checkpoint_idx,
            gamma: self.gamma,
            is_head: self.is_head,
            children: children,

            children_visit_count: self.children_visit_count.clone(),
            children_complete_visit_count: self.children_complete_visit_count.clone(),
            visited_node_count: self.visited_node_count,
            updated_node_count: self.updated_node_count,
        }
    }

    pub fn selection_action(&self) -> usize {
        let mut best_score = std::f32::MIN;
        let mut best_action = 0;

        for action in 0..(self.action_n as usize) {
            if self.children[action].is_none() {
                continue;
            }

            let exploit_score =
                self.q_value[action] / (self.children_complete_visit_count[action] as f32);
            let explore_score = f32::sqrt(
                2.0 * f32::ln(self.visit_count as f32) / (self.children_visit_count[action] as f32),
            );
            // TODO consider std?
            let score = exploit_score + 2.0 * explore_score;

            if score > best_score {
                best_score = score;
                best_action = action;
            }
        }
        best_action
    }

    pub fn update_history(&mut self, idx: u32, action_taken: usize, reward: f32) {
        self.traverse_history.insert(idx, (action_taken, reward));
    }

    pub fn add_child(
        &mut self,
        expand_action: usize,
        saving_idx: u32,
        gamma: f32,
        child_saturated: bool,
    ) {
        if child_saturated {
            assert!(self.is_head);
            // TODO no supported yet
        }

        match &self.children[expand_action] {
            None => {
                self.children[expand_action] =
                    Some(Node::new(self.action_n, saving_idx, gamma, false))
            }
            Some(child) => (), // FIXME this should be a bug?
        }
    }

    // pub fn child_ref(&mut self, action: usize) -> Result<&mut Box<Node>, ()> {
    //     match &mut self.children[action] {
    //         Some(e) => Ok(e),
    //         None => Err(()),
    //     }
    // }
    // pub fn parent_ref(&mut self) -> Result<&mut Box<Node>, ()> {
    //     match &mut self.parent {
    //         Some(e) => Ok(e),
    //         None => Err(()),
    //     }
    // }

    pub fn update_incomplete(&mut self, idx: u32) {}

    pub fn update_complete(&mut self, idx: u32, accu_reward: f32) {}
}

// impl PartialEq for Node {
//     fn eq(&self, other: &Self) -> bool {
//         self.checkpoint_idx == other.checkpoint_idx
//     }
// }
//
// impl Eq for Node {}

#[derive(Debug, Clone)]
pub struct NodeStub {
    pub action_n: u32,
    pub checkpoint_idx: u32,
    pub gamma: f32,
    pub is_head: bool,

    pub children: Vec<u32>,
    pub children_visit_count: Vec<u32>,
    pub children_complete_visit_count: Vec<u32>,
    pub visited_node_count: u32,
    pub updated_node_count: u32,
}

impl NodeStub {
    pub fn select_expansion_action(&self) -> usize {
        let mut cnt = 0;
        let mut rng = rand::thread_rng();
        let mut action: usize = 0;
        loop {
            if cnt < 20 {
                action = rng.gen_range(0..(self.action_n as usize));
            }

            if cnt > 100 {
                return action;
            }

            if self.children_visit_count[action] > 0 && cnt < 10 {
                cnt += 1;
                continue;
            }

            if self.children[action] == 0 {
                return action;
            }
        }
    }
}
