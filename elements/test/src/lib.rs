use hmny_common::prelude::*;

#[define_element{
    publisher: Publisher::new("Harmony", vec![]),
    element_type: ElementType::Test,
    common_query: match query {
        CommonQuery::Ping { message } => ping(message),
    }
}]
struct TestElement(CommonQuery);

fn ping(message: String) -> CommonResult {
    let response = format!(
        r#"Greetings "{}"! I am {}, the element. Pleasure to meet you :)"#,
        message, ELEMENT_NAME
    );

    Ok(CommonResponse::Pong { response })
}
