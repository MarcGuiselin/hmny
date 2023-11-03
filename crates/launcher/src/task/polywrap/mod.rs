pub use super::{Handle, Status, StatusSender};

#[derive(Clone, PartialEq, Debug, Eq)]
pub enum Polywrap {
    CompileWrap { name: String },
}

mod compile_wrap;

pub fn start_task(kind: Polywrap, send_task_update: StatusSender) -> Handle {
    match kind {
        Polywrap::CompileWrap { name } => compile_wrap::start_task(send_task_update, name),
    }
}
