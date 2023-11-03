#![allow(unused)]

pub mod wrap;
pub use wrap::prelude::*;

const WRAP_NAME: &str = env!("CARGO_CRATE_NAME");

impl ModuleTrait for Module {
    fn ping(args: ArgsPing) -> Result<PongResult, String> {
        Ok(PongResult {
            response: format!(
                r#"Greetings "{}"! I am {}, the wrap. Pleasure to meet you :)"#,
                args.message, WRAP_NAME
            ),
        })
    }
}
