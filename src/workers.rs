#[allow(unused_imports)]
use crate::eg_env::EgraphEnv;
#[allow(unused_imports)]
use crate::env::Env;
use crate::tree::{ExpTask, SimTask};

use egg::{Analysis, EGraph, Id, Language, RecExpr, Rewrite, StopReason};
use rand::Rng;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub enum Message<L, N>
where
    L: Language + 'static + egg::FromOp + std::marker::Send,
    N: Analysis<L> + Clone + 'static + std::default::Default + std::marker::Send,
    // N::Data: Clone
    N::Data: Clone,
    <N as Analysis<L>>::Data: Send,
{
    Exit,
    #[allow(unused_variables)]
    Nothing,
    Expansion(ExpTask<L, N>, u32, u32),
    Simulation(SimTask<L, N>, u32),
}

pub enum Reply<L, N>
where
    L: Language + 'static + egg::FromOp + std::marker::Send,
    N: Analysis<L> + Clone + 'static + std::default::Default + std::marker::Send,
    // N::Data: Clone
    N::Data: Clone,
    <N as Analysis<L>>::Data: Send,
{
    OK,
    DoneExpansion(
        usize,
        (),
        f32,
        bool,
        bool,
        Option<(u32, usize, EGraph<L, N>, Id, usize, usize)>,
        u32,
        u32,
    ),
    DoneSimulation(u32, f32),
}

pub fn worker_loop<L, N>(
    id: usize,
    gamma: f32,
    max_sim_step: u32,
    verbose: bool,
    expr: RecExpr<L>,
    rules: Vec<Rewrite<L, N>>,
    node_limit: usize,
    time_limit: usize,
) -> (
    thread::JoinHandle<()>,
    mpsc::Sender<Message<L, N>>,
    mpsc::Receiver<Reply<L, N>>,
)
where
    L: Language + 'static + egg::FromOp + std::marker::Send,
    N: Analysis<L> + Clone + 'static + std::default::Default + std::marker::Send,
    // N::Data: Clone
    N::Data: Clone,
    <N as Analysis<L>>::Data: Send,
{
    let (tx, rx) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    let handle = thread::spawn(move || {
        // make env
        // let mut env = Env::new(expr, rules, node_limit, time_limit);
        let mut env = EgraphEnv::new(expr, rules, node_limit, time_limit);
        env.reset();
        // worker loop
        loop {
            let message = rx.recv().unwrap();
            match message {
                Message::Exit => {
                    println!("Worker {} Exit!", id);
                    break;
                }

                Message::Expansion(exp_task, global_saving_idx, task_idx) => {
                    if verbose {
                        println!("Worker {} Expansion!", id);
                    }
                    // expand one step
                    env.restore(exp_task.checkpoint_data);
                    let expand_action = exp_task.shallow_copy_node.select_expansion_action();
                    let (next_state, reward, done, info) = env.step(expand_action);
                    let new_checkpoint_data;
                    if done {
                        new_checkpoint_data = None;
                    } else {
                        new_checkpoint_data = Some(env.checkpoint());
                    }
                    let mut child_saturated = false;
                    if exp_task.shallow_copy_node.is_head {
                        match info.report.stop_reason {
                            StopReason::Saturated => child_saturated = true,
                            _ => (),
                        }
                    }

                    // reply
                    tx2.send(Reply::DoneExpansion(
                        expand_action,
                        next_state,
                        reward,
                        done,
                        child_saturated,
                        new_checkpoint_data,
                        global_saving_idx,
                        task_idx,
                    ))
                    .unwrap();
                }

                Message::Simulation(sim_task, task_idx) => {
                    env.restore(sim_task.checkpoint_data);
                    assert!(sim_task.action_applied);

                    let mut cnt = 0;
                    let mut _state;
                    let mut reward;
                    let mut done = false; // NOTE if already done, then this simulation will not be scheduled
                    let mut accu_reward = 0.0;
                    let mut accu_gamma = 1.0;
                    let mut _info;
                    // start_state_value = self.get_value(_state) // TODO
                    let start_state_value = 0.0; // to tune?
                    let factor = 1.0; //  to tune?
                    let mut rng = rand::thread_rng();

                    // env loop
                    while !done {
                        // random policy rollouts
                        let action_n = env.get_action_space();
                        let action = rng.gen_range(0..action_n);
                        (_state, reward, done, _info) = env.step(action);

                        // timeLimited truncate
                        if cnt == max_sim_step && !done {
                            done = true;
                            // get the final reward
                            // reward = env.get_reward();
                        }

                        accu_reward += reward * accu_gamma;
                        accu_gamma *= gamma;
                        cnt += 1;
                    }

                    //  Use V(s) to stabilize simulation return
                    accu_reward = accu_reward * factor + start_state_value * (1.0 - factor);

                    // reply
                    tx2.send(Reply::DoneSimulation(task_idx, accu_reward))
                        .unwrap();
                }

                Message::Nothing => {
                    // act as random straggler
                    let mut rng = rand::thread_rng();
                    thread::sleep(Duration::from_secs(rng.gen_range(0..5)));
                    tx2.send(Reply::OK).unwrap();
                }
            }
        }
        println!("Worker {} Exit successfully!", id);
    });

    (handle, tx, rx2)
}
