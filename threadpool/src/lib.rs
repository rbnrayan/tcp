use std::thread;
use std::sync::{
    mpsc,
    Arc,
    Mutex
};

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Clone, Copy)]
pub enum Mode {
    Quiet,
    Verbose,
}

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
    mode: Mode
}

impl ThreadPool {
    pub fn new(size: usize, mode: Mode) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver), mode));
        }

        ThreadPool {
            workers,
            sender,
            mode,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static
    {
        let job = Message::NewJob(Box::new(f));
        self.sender.send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if let Mode::Verbose = self.mode {
            println!("Sending terminate to all workers");
        }

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        if let Mode::Verbose = self.mode {
            println!("Shuttind down all workers");
        }

        for worker in &mut self.workers {
            if let Mode::Verbose = self.mode {
                println!("Shutting down worker {}", worker.id);
            }

            if let Some(thread) = worker.thread.take() {
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
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>, mode: Mode) -> Self {
        let thread = Some(thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    if let Mode::Verbose = mode {
                        println!("Worker {} got a job; executing", id);
                    }
                    job();
                }
                Message::Terminate => {
                    if let Mode::Verbose = mode {
                        println!("Worker {} was told to terminate", id);
                    }    
                    break;
                }
            }
        }));

        Worker {
            id,
            thread,
        }
    }
}
