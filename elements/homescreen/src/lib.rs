use hmny_common::prelude::*;

define_element! {
    publisher: Publisher::new("Harmony", vec![]),
    element_type: ElementType::HomeScreen,
    signals: match signal {
        Signal::AskHomeScreen => home_screen(),
    }
}

fn home_screen() -> SignalResult {
    Ok(Signal::HomeScreen {
        mime_type: "txt".into(),
        data: DataType::String(include_str!("homescreen.md").into()),
    })
}
