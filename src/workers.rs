use std::sync::mpsc;
use std::thread;

pub enum Message {
    Exit,
    Nothing,
    Expansion,
    Simulation,
}

pub fn worker_loop() -> (
    thread::JoinHandle<()>,
    mpsc::Sender<Message>,
    mpsc::Receiver<Message>,
) {
    let (tx, rx) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    let handle = thread::spawn(move || {
        // worker loop
        loop {
            let message = rx.recv().unwrap();
            match message {
                Message::Exit => {
                    println!("Exit!");
                    break;
                }
                Message::Nothing => {
                    println!("Nothing!");
                }

                Message::Expansion => {
                    println!("Expansion!");
                }

                Message::Simulation => {
                    println!("Simulation!");
                }
            }
        }
    });

    (handle, tx, rx2)
}
