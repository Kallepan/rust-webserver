use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::info;

pub struct ThreadPool {
    _workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
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
            _workers: workers,
            sender,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

struct Worker {
    _id: usize,
    _thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            info!("Worker {} got a job; executing.", id);
            job();
        });

        Worker {
            _id: id,
            _thread: thread,
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

    worker._thread.join().unwrap();
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
