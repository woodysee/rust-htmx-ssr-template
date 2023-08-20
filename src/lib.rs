use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    sender: mpsc::Sender<Signal>,
    workers: Vec<Worker>,
}

type Task = Box<dyn FnOnce() + Send + 'static>;

enum Signal {
    Dispatch(Task),
    Drop,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// # Parameters
    /// - `size`: number of threads in the pool. Panics when `size == 0`.
    ///
    pub fn new(size: usize) -> ThreadPool {
        // Ensure panic for size
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let task = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&task)));
        }

        ThreadPool { sender, workers }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = Box::new(f);
        self.sender.send(Signal::Dispatch(task)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending drop signal to all workers.");

        for _ in &self.workers {
            self.sender.send(Signal::Drop).unwrap();
        }

        println!("Dropping all workers.");

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                println!("Worker {}: Joining...", worker.id);
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Signal>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let signal = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {} got a task; executing.", id);
            match signal {
                Signal::Dispatch(run_task) => {
                    println!("Worker {} got a task; executing.", id);
                    run_task();
                }
                Signal::Drop => {
                    println!("Worker {} was told to drop task. Exiting loop.", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
