use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self},
};

use crate::info;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        info!("Sending terminate message to all workers.");

        for worker in &mut self.workers {
            info!("Shutting down worker {}", worker._id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    info!("Worker {} got a job; executing.", id);
                    job();
                }
                Err(_) => {
                    info!("Worker {} is shutting down.", id);
                    break;
                }
            }
        });

        Worker {
            _id: id,
            thread: Some(thread),
        }
    }
}

#[test]
fn test_worker() {
    let (sender, receiver) = mpsc::channel();

    let receiver = Arc::new(Mutex::new(receiver));

    let worker = Worker::new(0, Arc::clone(&receiver));

    sender
        .send(Box::new(|| {
            info!("This is a test job.");
        }))
        .unwrap();
    sender
        .send(Box::new(|| {
            info!("This is a test job.");
        }))
        .unwrap();

    // stop the worker by dropping the sender
    drop(sender);

    worker.thread.unwrap().join().unwrap();
}

#[test]
fn test_thread_pool() {
    let pool = ThreadPool::new(4);

    for i in 0..8 {
        pool.execute(move || {
            info!("Task {} is running.", i);
        });
    }
}
