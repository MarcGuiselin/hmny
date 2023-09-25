pub use super::{Cargo, Ready, Status, StatusSender};

mod command;
pub use command::{package_dependency_count, CargoCommand};

mod build_wraps;

pub fn create_ready(kind: Cargo, send_task_update: StatusSender) -> Ready {
    match kind {
        Cargo::BuildWraps => build_wraps::create_ready(send_task_update),
        _ => unimplemented!(),
    }
}
