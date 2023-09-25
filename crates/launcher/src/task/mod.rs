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

pub type Active = Box<dyn Send + Sync>;

/// A task that is ready to be turned into an active task
pub struct Ready {
    id: Uuid,
    active: Active,
}
impl Ready {
    pub fn new(id: Uuid, active: Active) -> Self {
        Self { id, active }
    }
}

impl Task {
    pub fn into_active(self, send_task_update: mpsc::Sender<Status>) -> ActiveTask {
        let Ready { id, active } = match self.clone() {
            Task::Dev(dev) => match dev {
                Dev::Cargo(kind) => cargo::create_ready(kind, send_task_update),
                Dev::CompileWrap { name } => unimplemented!(),
            },
        };

        ActiveTask {
            id,
            task: self,
            active,
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

pub struct ActiveTask {
    pub id: Uuid,
    pub task: Task,
    active: Active,
}
