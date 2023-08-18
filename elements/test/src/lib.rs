use hmny_common::prelude::*;

define_element! {
    publisher: Publisher::new("Harmony", vec![]),
    element_type: ElementType::Test,
    signals: match signal {
        Signal::Ping { message } => {
            let response = format!(
                r#"Greetings "{}"! I am {}, the element. Pleasure to meet you :)"#,
                message, ELEMENT_NAME
            );

            Ok(Signal::Pong { response })
        }
    }
}
