use crate::tree::{ExpTask, SimTask};
use crate::workers::{worker_loop, Message, Reply};

use std::sync::mpsc::{Receiver, Sender};
use std::thread;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    Busy,
    Idle,
}

pub struct PoolManager {
    name: &'static str,
    work_num: u32, // TODO determine this automatically
    gamma: f32,
    max_sim_step: u32,

    // self
    workers: Vec<thread::JoinHandle<()>>,
    worker_status: Vec<Status>,
    txs: Vec<Sender<Message>>,
    rxs: Vec<Receiver<Reply>>,
}

impl PoolManager {
    pub fn new(
        name: &'static str,
        work_num: u32,
        gamma: f32,
        max_sim_step: u32,
        verbose: bool,
    ) -> Self {
        // build workers
        let mut workers = Vec::new();
        let mut txs = Vec::new();
        let mut rxs = Vec::new();
        for i in 0..work_num {
            let (w, tx, rx) = worker_loop(i, gamma, max_sim_step, verbose);
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
        self.worker_status.contains(&Status::Idle)
    }

    pub fn assign_expansion_task(
        &mut self,
        exp_task: ExpTask,
        global_saving_idx: u32,
        task_idx: u32,
    ) {
        let id = self.find_idle_worker();
        self.txs[id]
            .send(Message::Expansion(exp_task, global_saving_idx, task_idx))
            .unwrap();
    }

    pub fn assign_simulation_task(
        &mut self,
        sim_task: SimTask,
        sim_checkpoint_data: Vec<u32>,
        task_idx: u32,
    ) {
        let id = self.find_idle_worker();
        self.txs[id]
            .send(Message::Simulation(sim_task, sim_checkpoint_data, task_idx))
            .unwrap();
    }

    pub fn assign_nothing_task(&mut self) {
        let id = self.find_idle_worker();
        self.txs[id].send(Message::Nothing).unwrap();
    }

    fn find_idle_worker(&mut self) -> usize {
        for (i, status) in self.worker_status.iter_mut().enumerate() {
            match status {
                Status::Busy => (),
                Status::Idle => {
                    self.worker_status[i] = Status::Busy;
                    return i;
                }
            }
        }
        panic!("no idle worker");
    }

    pub fn occupancy(&mut self) -> f32 {
        (self
            .worker_status
            .iter()
            .fold(0, |acc, x| if x == &Status::Busy { acc + 1 } else { acc }) as f32)
            / (self.work_num as f32)
    }

    pub fn get_complete_task(&mut self) -> Reply {
        loop {
            for i in 0..(self.work_num as usize) {
                let reply = self.rxs[i].try_recv(); // non-blocking
                match reply {
                    Err(_) => (),
                    Ok(r) => {
                        self.worker_status[i] = Status::Idle;
                        return r;
                    }
                }
            }
        }
    }

    pub fn close(&mut self) {
        // wait until all exit
        for id in 0..(self.work_num as usize) {
            match self.worker_status[id] {
                Status::Idle => self.txs[id].send(Message::Exit).unwrap(),
                Status::Busy => {
                    self.rxs[id].recv().unwrap(); // block until workers finish
                    self.txs[id].send(Message::Exit).unwrap();
                    self.worker_status[id] = Status::Idle;
                }
            }
        }
        // join
        while self.workers.len() > 0 {
            let w = self.workers.remove(0); // get ownership
            w.join().unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(unused_imports)]
    // use super::{PoolManager, worker_loop};
    use super::*;
    // use std::sync::atomic::{AtomicUsize, Ordering};
    // use std::sync::mpsc::{channel, sync_channel};
    // use std::sync::{Arc, Barrier};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_build_and_close() {
        let mut pool = PoolManager::new("test", 1, 1.0, 1, true);
        assert_eq!(pool.has_idle_server(), true);
        pool.assign_nothing_task();
        println!("occupancy: {}", pool.occupancy());

        // let a = vec![1, 2, 3];
        // for (i, j) in a.iter().enumerate() {
        //     println!("{} - {}", i, j);
        // }
        thread::sleep(Duration::from_secs(1));

        pool.close();
    }

    #[test]
    fn test_poll_channel() {
        let worker_num = 5;
        let mut pool = PoolManager::new("test", worker_num, 1.0, 1, true);
        pool.assign_nothing_task();
        pool.assign_nothing_task();
        pool.assign_nothing_task();
        pool.assign_nothing_task();

        println!("occupancy: {}", pool.occupancy());
        pool.close();
        println!("test_pool done");
    }
}
