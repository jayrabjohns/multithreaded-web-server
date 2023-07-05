use std::{
    io,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use super::thread_pool::Job;

pub struct Worker {
    pub id: usize,
    pub handler: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> io::Result<Worker> {
        let builder = thread::Builder::new();
        let handler = builder.spawn(move || loop {
            let message = receiver.lock().expect("Mutex is in poisened state. It's possible this is caused by other threads panicking.").recv();
            match message {
                Ok(job) => {
                    println!("[Worker {id}] Starting execution of new job...");
                    job();
                    println!("[Worker {id}] Finished execution of job. Waiting for new work...");
                }
                Err(_) => {
                    println!("[Worker {id}] Disconnected. Shutting down.");
                    break;
                }
            }
        })?;

        Ok(Worker {
            id,
            handler: Some(handler),
        })
    }
}
