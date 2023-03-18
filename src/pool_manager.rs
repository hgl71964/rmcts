use crate::tree::{ExpTask, SimTask};
use crate::workers::{worker_loop, Message};

use std::sync::mpsc::{Receiver, Sender};
use std::thread;

#[derive(Debug, Clone)]
enum Status {
    Busy,
    Idle,
}

pub struct PoolManager {
    name: &'static str,
    work_num: u32,
    gamma: f32,
    max_sim_step: u32,

    // self
    workers: Vec<thread::JoinHandle<()>>,
    worker_status: Vec<Status>,
    txs: Vec<Sender<Message>>,
    rxs: Vec<Receiver<Message>>,
}

impl PoolManager {
    pub fn new(name: &'static str, work_num: u32, gamma: f32, max_sim_step: u32) -> Self {
        // build workers
        let mut workers = Vec::new();
        let mut txs = Vec::new();
        let mut rxs = Vec::new();
        for i in 0..work_num {
            let (w, tx, rx) = worker_loop();
            workers.push(w);
            txs.push(tx);
            rxs.push(rx);
        }

        PoolManager {
            name: name,
            work_num: work_num,
            gamma: gamma,
            max_sim_step: max_sim_step,
            workers: workers,
            worker_status: vec![Status::Idle; usize::try_from(work_num).unwrap()],
            txs: txs,
            rxs: rxs,
        }
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

    pub fn assign_simulation_task(&mut self, sim_task: SimTask, idx: u32) {}

    fn find_idle_worker(&mut self) -> usize {
        for (i, status) in self.worker_status.iter_mut().enumerate() {
            match status {
                Status::Idle => {
                    self.worker_status[i] = Status::Busy;
                    return i;
                }

                Status::Busy => {}
            }
        }
        panic!("no idle worker");
    }

    pub fn occupancy(&mut self) -> f32 {
        1.0
    }

    pub fn get_complete_expansion_task(&mut self) {}

    fn wait_until_all_idle(&mut self) {
        for id in 0..(self.work_num as usize) {
            match self.worker_status[id] {
                Status::Idle => (),
                Status::Busy => {
                    self.txs[id].send(Message::Exit).unwrap();
                    self.worker_status[id] = Status::Idle;
                }
            }
        }
    }

    pub fn close(&mut self) {
        self.wait_until_all_idle();
        while self.workers.len() > 0 {
            let w = self.workers.remove(0); // get ownership
            w.join().unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    // use super::{PoolManager, worker_loop};
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::{channel, sync_channel};
    use std::sync::{Arc, Barrier};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_build_and_close() {
        let mut pool = PoolManager::new("test", 1, 1.0, 1);

        // TODO send a message to worker so it becomes Busy first

        let a = vec![1, 2, 3];
        for (i, j) in a.iter().enumerate() {
            println!("{} - {}", i, j);
        }
        thread::sleep(Duration::from_secs(1));

        pool.close();
    }
}
