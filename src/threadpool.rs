use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Message>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "ThreadPool må ha minst 1 tråd");

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
        if let Some(ref sender) = self.sender {
            let _ = sender.send(Message::NewJob(job));
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            for _ in &self.workers {
                let _ = sender.send(Message::Terminate);
            }
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = {
                let lock = receiver.lock().unwrap();
                lock.recv()
            };

            match message {
                Ok(Message::NewJob(job)) => {
                    job();
                }
                Ok(Message::Terminate) | Err(_) => {
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
