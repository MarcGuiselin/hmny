use tokio::sync::mpsc;
use uuid::{uuid, Uuid};

#[derive(Clone, PartialEq, Debug, Eq)]
pub enum Task {
    Dev(Dev),
    // Prod(Prod),
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub enum Dev {
    Cargo(Cargo),
    CompileWrap { name: String },
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub enum Cargo {
    CleanWraps,
    BuildWraps,
}

pub type StatusSender = mpsc::Sender<Status>;
pub type StatusReceiver = mpsc::Receiver<Status>;

impl Task {
    pub fn into_active(self, send_task_update: mpsc::Sender<Status>) -> ActiveTask {
        match self {
            Task::Dev(dev) => match dev {
                Dev::Cargo(Cargo::CleanWraps) => unimplemented!(),
                Dev::Cargo(Cargo::BuildWraps) => unimplemented!(),
                Dev::CompileWrap { name } => unimplemented!(),
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Status {
    pub id: Uuid,
    pub title: String,
    pub status: Option<String>,
    pub done_ratio: f32,
    pub doing_ratio: f32,
    pub error: Option<String>,
}

impl PartialEq for Status {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Status {
    pub fn is_completed(&self) -> bool {
        self.done_ratio >= 1.0
    }
}

pub trait Active: Send + Sync {
    fn get_status(&self) -> Status;
}

pub struct ActiveTask {
    task: Task,
    active: Box<dyn Active>,
    pub status: Status,
}
