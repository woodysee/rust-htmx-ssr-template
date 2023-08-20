use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    sender: mpsc::Sender<Task>,
    #[allow(dead_code)]
    workers: Vec<Worker>
}

type Task = Box<dyn FnOnce() + Send + 'static>;

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
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

struct Worker {
    #[allow(dead_code)]
    id: usize,
    #[allow(dead_code)]
    thread: thread::JoinHandle<()>
}

impl Worker  {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Task>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let run_task = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {} got a task; executing.", id);
            run_task();
        });
        Worker { id, thread }
    }
}
