use crate::env::Env;
use crate::tree::{ExpTask, SimTask};
use std::sync::mpsc;
use std::thread;

pub enum Message {
    Exit,
    Nothing,
    Expansion(ExpTask, u32, u32),
    Simulation(SimTask),
}

pub enum Reply {
    OK,
    DoneExpansion(usize, u32, f32, bool, bool, Option<Vec<u32>>, u32, u32),
    DoneSimulation(),
}

pub fn worker_loop(
    id: u32,
    gamma: f32,
    max_sim_step: u32,
    verbose: bool,
) -> (
    thread::JoinHandle<()>,
    mpsc::Sender<Message>,
    mpsc::Receiver<Reply>,
) {
    let mut env = Env::new();

    let (tx, rx) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    let handle = thread::spawn(move || {
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
                    let mut new_checkpoint_data: Option<Vec<u32>>;
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

                Message::Simulation(sim_task) => {
                    println!("Simulation!");
                    tx2.send(Reply::DoneSimulation()).unwrap();
                }

                Message::Nothing => tx2.send(Reply::OK).unwrap(),
            }
        }
        println!("Worker {} Exit successfully!", id);
    });

    (handle, tx, rx2)
}
