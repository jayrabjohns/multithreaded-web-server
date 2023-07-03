use std::{
    io,
    sync::{mpsc, Arc, Mutex},
};

use super::worker::Worker;

pub struct ThreadPool {
    _workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

pub type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Creates a new ThreadPool.
    ///
    /// `size`: The number of underlying threads in the pool.
    ///
    /// Panics if size is 0.
    pub fn new(size: usize) -> io::Result<ThreadPool> {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver))?)
        }

        Ok(ThreadPool {
            _workers: workers,
            sender,
        })
    }

    /// Executes a given closure on a worker thread.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap(); // Should never panic
    }
}
