pub use super::{Cargo, Handle, Status, StatusSender};

mod command;
pub use command::{package_dependency_count, CargoCommand};

mod build_wraps;

pub fn start_task(kind: Cargo, send_task_update: StatusSender) -> Handle {
    match kind {
        Cargo::BuildWraps => build_wraps::start_task(send_task_update),
        _ => unimplemented!(),
    }
}
