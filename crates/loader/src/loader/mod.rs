use std::{
    sync::{mpsc, Arc, Mutex},
    thread, vec,
};

mod wrap;

pub trait Loader: Send + Sync {
    fn get_status(&self) -> TaskStatus;
}

pub type UpdateSender = mpsc::Sender<()>;
pub type UpdateReceiver = mpsc::Receiver<()>;

#[allow(dead_code)]
pub struct Loaders {
    active: Arc<Vec<Box<dyn Loader>>>,
    update_sender: UpdateSender,
    update_receiver: Arc<Mutex<UpdateReceiver>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskStatus {
    pub title: String,
    pub status: Option<String>,
    pub done_ratio: f32,
    pub doing_ratio: f32,
    pub error: Option<String>,
}

#[allow(dead_code)]
impl TaskStatus {
    pub fn is_completed(&self) -> bool {
        self.done_ratio == 1.0
    }
}

#[allow(dead_code)]
impl Loaders {
    pub fn init() -> Self {
        let (update_sender, update_receiver) = mpsc::channel();
        let update_receiver = Arc::new(Mutex::new(update_receiver));

        let active: Vec<Box<dyn Loader>> = vec![
            // Start compiling and loading wraps
            wrap::WrapLoader::new_boxed(&update_sender),
        ];

        Self {
            active: Arc::new(active),
            update_sender,
            update_receiver,
        }
    }

    pub fn status(&self) -> Vec<TaskStatus> {
        get_status(&self.active)
    }

    pub fn subscribe<F>(&self, mut callback: F)
    where
        F: FnMut(Vec<TaskStatus>) + Send + 'static,
    {
        let update_receiver = Arc::clone(&self.update_receiver);
        let active = Arc::clone(&self.active);

        callback(get_status(&active));
        thread::spawn(move || loop {
            // Lock the receiver and wait for a message
            let lock = update_receiver.lock().unwrap();
            match lock.recv() {
                Ok(_) => callback(get_status(&active)),
                Err(_) => break, // Handle disconnected sender
            }
        });
    }
}

pub fn get_status(loaders: &Vec<Box<dyn Loader>>) -> Vec<TaskStatus> {
    loaders.iter().map(|loader| loader.get_status()).collect()
}
