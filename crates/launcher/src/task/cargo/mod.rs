pub use super::{Handle, Status, StatusSender};

mod command;
pub use command::{package_dependency_count, CargoCommand};

#[derive(Clone, PartialEq, Debug, Eq)]
pub enum Cargo {
    CleanWraps,
    BuildWraps,
}

mod build_wraps;

pub fn start_task(kind: Cargo, send_task_update: StatusSender) -> Handle {
    match kind {
        Cargo::BuildWraps => build_wraps::start_task(send_task_update),
        _ => unimplemented!(),
    }
}
