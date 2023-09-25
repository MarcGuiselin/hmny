use tokio::sync::mpsc;
use uuid::Uuid;

mod cargo;

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

/// A task that is ready to be turned into an active task
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Handle(Uuid);

impl Handle {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Task {
    pub fn into_active(self, send_task_update: mpsc::Sender<Status>) -> ActiveTask {
        let handle = match self.clone() {
            Task::Dev(dev) => match dev {
                Dev::Cargo(kind) => cargo::start_task(kind, send_task_update),
                Dev::CompileWrap { name } => unimplemented!(),
            },
        };

        ActiveTask { handle, task: self }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Status {
    pub handle: Handle,
    pub title: String,
    pub status: Option<String>,
    pub done_ratio: f32,
    pub doing_ratio: f32,
    pub error: Option<String>,
}

impl PartialEq for Status {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl Status {
    pub fn is_completed(&self) -> bool {
        self.done_ratio >= 1.0
    }
}

pub struct ActiveTask {
    pub handle: Handle,
    pub task: Task,
}
