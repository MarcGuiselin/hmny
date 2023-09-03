use hmny_common::prelude::*;

#[define_wrap{
    publisher: Publisher::new("Harmony", vec![]),
    wrap_type: WrapType::Test,
    common_query: match query {
        CommonQuery::Ping { message } => ping(message),
    }
}]
struct TestWrap(CommonQuery);

fn ping(message: String) -> CommonResult {
    let response = format!(
        r#"Greetings "{}"! I am {}, the wrap. Pleasure to meet you :)"#,
        message, WRAP_NAME
    );

    Ok(CommonResponse::Pong { response })
}
