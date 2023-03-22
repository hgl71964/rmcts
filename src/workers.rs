use crate::env::Env;
use crate::tree::{ExpTask, SimTask};

use egg::{Analysis, Language, Rewrite};
use rand::Rng;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub enum Message {
    Exit,
    #[allow(unused_variables)]
    Nothing,
    Expansion(ExpTask, u32, u32),
    Simulation(SimTask, Vec<usize>, u32),
}

pub enum Reply {
    OK,
    DoneExpansion(usize, (), f32, bool, bool, Option<Vec<usize>>, u32, u32),
    DoneSimulation(u32, f32),
}

pub fn worker_loop<
    L: Language + 'static + egg::FromOp,
    N: Analysis<L> + Clone + 'static + std::default::Default,
>(
    id: usize,
    gamma: f32,
    max_sim_step: u32,
    verbose: bool,
    expr: &'static str,
    rules: Vec<Rewrite<L, N>>,
    node_limit: usize,
    time_limit: usize,
) -> (
    thread::JoinHandle<()>,
    mpsc::Sender<Message>,
    mpsc::Receiver<Reply>,
) {
    let (tx, rx) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    let handle = thread::spawn(move || {
        // make env
        let mut env = Env::new(expr, rules, node_limit, time_limit);
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
                    let new_checkpoint_data: Option<Vec<usize>>;
                    if done {
                        new_checkpoint_data = None;
                    } else {
                        new_checkpoint_data = Some(env.checkpoint());
                    }

                    // TODO
                    let child_saturated = false;
                    // if exp_task.shallow_copy_node.is_head && info["stop_reason"] == "SATURATED":
                    //     child_saturated = True

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

                Message::Simulation(sim_task, sim_checkpoint_data, task_idx) => {
                    env.restore(sim_checkpoint_data);
                    assert!(sim_task.action_applied);

                    let mut cnt = 0;
                    let mut state;
                    let mut reward;
                    let mut done = false; // NOTE if already done, then this simulation will not be scheduled
                    let mut accu_reward = 0.0;
                    let mut accu_gamma = 1.0;
                    let mut info;
                    // start_state_value = self.get_value(state) // TODO
                    let start_state_value = 0.0; // to tune?
                    let factor = 1.0; //  to tune?
                    let mut rng = rand::thread_rng();

                    // env loop
                    while !done {
                        // random policy rollouts
                        let action_n = env.get_action_space();
                        let action = rng.gen_range(0..action_n);
                        (state, reward, done, info) = env.step(action);

                        // timeLimited truncate
                        if cnt == max_sim_step && !done {
                            done = true;
                            // get the final reward TODO
                            // reward = env.reward_func(
                            // 	done, info, self.wrapped_env.egraph, self.wrapped_env.expr,
                            // 	self.wrapped_env.base_cost)
                            reward = 0.0;
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
