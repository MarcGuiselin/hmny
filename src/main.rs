#[derive(serde::Serialize)]
pub struct PingArgs {
    pub message: String,
}

#[derive(serde::Deserialize)]
pub struct PongResult {
    pub response: String,
}

fn main() {
    use polywrap::*;

    let start = std::time::Instant::now();
    let mut config = ClientConfig::new();
    config.add(SystemClientConfig::default().into());
    let client = Client::new(config.build());
    println!("Polywrap client created in {:?}", start.elapsed());

    let start = std::time::Instant::now();
    let add_file_resp = client
        .invoke::<PongResult>(
            &Uri::try_from("wrap://fs/C:/Users/Marc/Documents/Projects/Harmony/hmny/target/polywrap/release/test_wrap").unwrap(),
            "ping",
            Some(
                &to_vec(&PingArgs {
                    message: "Hello from Rust!".to_string(),
                })
                .unwrap(),
            ),
            None,
            None,
        )
        .unwrap();
    println!("Resolving wrap and invoking took {:?}", start.elapsed());
    println!("Response to ping: '{}'", add_file_resp.response);

    let start = std::time::Instant::now();
    let add_file_resp = client
        .invoke::<PongResult>(
            &Uri::try_from("wrap://fs/C:/Users/Marc/Documents/Projects/Harmony/hmny/target/polywrap/release/test_wrap").unwrap(),
            "ping",
            Some(
                &to_vec(&PingArgs {
                    message: "Hello from Rust 2!".to_string(),
                })
                .unwrap(),
            ),
            None,
            None,
        )
        .unwrap();
    println!("Resolving wrap and invoking 2 took {:?}", start.elapsed());
    println!("Response to ping 2: '{}'", add_file_resp.response);

    let start = std::time::Instant::now();
    let add_file_resp = client
        .invoke::<PongResult>(
            &Uri::try_from("wrap://fs/C:/Users/Marc/Documents/Projects/Harmony/hmny/target/polywrap/release/test_wrap").unwrap(),
            "ping",
            Some(
                &to_vec(&PingArgs {
                    message: "Hello from Rust 2!".to_string(),
                })
                .unwrap(),
            ),
            None,
            None,
        )
        .unwrap();
    println!("Resolving wrap and invoking 3 took {:?}", start.elapsed());
    println!("Response to ping 3: '{}'", add_file_resp.response);
}
